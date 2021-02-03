use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    internet::{ip, InternetProtocol, InternetProtocolError},
    link::{self, ethernet, LinkProtocol},
    network_device, option, RxResult,
};

use super::{ARPHeader, Operation};

pub fn rx<ND: 'static + network_device::NetworkDevice>(
    opt: Arc<option::PeachPSOption>,
    dev: Arc<Mutex<ND>>,
    mut rx_result: RxResult,
    buf: &[u8],
    arp_cache: Arc<Mutex<HashMap<ip::IPv4Addr, link::MacAddress>>>,
) -> Result<(RxResult, Vec<u8>), InternetProtocolError> {
    let arp_packet_hdr =
        ARPHeader::new_from_bytes(buf, InternetProtocolError::CannotParsePacketHeader)?;
    let (_, rest) = buf.split_at(ARPHeader::LENGTH);

    if arp_packet_hdr.operation == Operation::Request {
        // ARPテーブルのロックをとって書き込む
        arp_cache.lock().unwrap().insert(
            arp_packet_hdr.src_internet_addr,
            arp_packet_hdr.src_link_addr,
        );
        if opt.debug {
            eprintln!(
                "inserting an ARP table entry `{} => {}`",
                arp_packet_hdr.src_internet_addr, arp_packet_hdr.src_link_addr
            );
        }

        std::thread::spawn(move || {
            tx_reply(opt, dev, &arp_packet_hdr).unwrap();
        })
        .join()
        .unwrap();
    }

    rx_result.src_ip_addr = arp_packet_hdr.src_internet_addr;

    Ok((rx_result, rest.to_vec()))
}

pub fn tx_request<ND: network_device::NetworkDevice>(
    opt: Arc<option::PeachPSOption>,
    dev: Arc<Mutex<ND>>,
    target_ip: ip::IPv4Addr,
) -> Result<(), InternetProtocolError> {
    let mut send_arp_packet: ARPHeader = Default::default();
    send_arp_packet.operation = Operation::Request;
    send_arp_packet.src_internet_addr = opt.ip_addr;
    send_arp_packet.src_link_addr = opt.dev_addr;
    send_arp_packet.dst_internet_addr = target_ip;
    send_arp_packet.link_type = LinkProtocol::Ethernet;
    send_arp_packet.internet_type = InternetProtocol::IP;
    send_arp_packet.internet_addr_length = 6;
    send_arp_packet.link_addr_length = 4;

    if opt.debug {
        eprintln!("++++++++ tx arp packet ++++++++");
        eprintln!("{}", send_arp_packet);
    }

    ethernet::tx(
        opt,
        dev,
        InternetProtocol::ARP,
        link::BLOADCAST_MAC_ADDRESS,
        send_arp_packet.to_bytes(InternetProtocolError::CannotConstructPacket)?,
    )?;

    Ok(())
}

fn tx_reply<ND: network_device::NetworkDevice>(
    opt: Arc<option::PeachPSOption>,
    dev: Arc<Mutex<ND>>,
    receive_arp_packet: &ARPHeader,
) -> Result<(), InternetProtocolError> {
    tx(opt, dev, Operation::Reply, receive_arp_packet)
}

fn tx<ND: network_device::NetworkDevice>(
    opt: Arc<option::PeachPSOption>,
    dev: Arc<Mutex<ND>>,
    op: Operation,
    receive_arp_packet: &ARPHeader,
) -> Result<(), InternetProtocolError> {
    let mut send_arp_packet: ARPHeader = Default::default();
    // 殆どの部分は，受信したパケットの値をコピーするだけで良い
    send_arp_packet.link_type = receive_arp_packet.link_type;
    send_arp_packet.internet_type = receive_arp_packet.internet_type;
    send_arp_packet.internet_addr_length = receive_arp_packet.internet_addr_length;
    send_arp_packet.link_addr_length = receive_arp_packet.link_addr_length;
    // srcとdstの関係が逆になるので注意．
    send_arp_packet.dst_link_addr = receive_arp_packet.src_link_addr;
    send_arp_packet.dst_internet_addr = receive_arp_packet.src_internet_addr;

    send_arp_packet.operation = op;

    // 自身のアドレスを書き込んで教える
    send_arp_packet.src_link_addr = opt.dev_addr;
    send_arp_packet.src_internet_addr = opt.ip_addr;

    if opt.debug {
        eprintln!("++++++++ tx arp packet ++++++++");
        eprintln!("{}", send_arp_packet);
    }

    ethernet::tx(
        opt,
        dev,
        InternetProtocol::ARP,
        send_arp_packet.dst_link_addr,
        send_arp_packet.to_bytes(InternetProtocolError::CannotConstructPacket)?,
    )?;

    Ok(())
}
