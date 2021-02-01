use crate::{
    internet::{InternetProtocol, InternetProtocolError},
    link::ethernet,
    network_device, option, RxResult,
};

use super::{ARPHeader, Operation};

pub fn rx<ND: network_device::NetworkDevice>(
    opt: option::PeachPSOption,
    dev: &mut ND,
    _rx_result: RxResult,
    buf: &[u8],
) -> Result<(RxResult, Vec<u8>), InternetProtocolError> {
    let arp_packet_hdr =
        ARPHeader::new_from_bytes(buf, InternetProtocolError::CannotParsePacketHeader)?;

    if opt.debug {
        eprintln!("++++++++ ARP Packet ++++++++");
        eprintln!("{}", arp_packet_hdr);
    }

    if arp_packet_hdr.operation == Operation::Request {
        tx_reply(opt, dev, &arp_packet_hdr)?;
    }

    todo!()
}

#[allow(dead_code)]
fn tx_request<ND: network_device::NetworkDevice>(
    opt: option::PeachPSOption,
    dev: &mut ND,
    receive_arp_packet: &ARPHeader,
) -> Result<(), InternetProtocolError> {
    tx(opt, dev, Operation::Request, receive_arp_packet)
}

fn tx_reply<ND: network_device::NetworkDevice>(
    opt: option::PeachPSOption,
    dev: &mut ND,
    receive_arp_packet: &ARPHeader,
) -> Result<(), InternetProtocolError> {
    tx(opt, dev, Operation::Reply, receive_arp_packet)
}

fn tx<ND: network_device::NetworkDevice>(
    opt: option::PeachPSOption,
    dev: &mut ND,
    op: Operation,
    receive_arp_packet: &ARPHeader,
) -> Result<(), InternetProtocolError> {
    let mut send_arp_packet: ARPHeader = Default::default();
    // 殆どの部分は，受信したパケットの値をコピーするだけで良い
    send_arp_packet.link_type = receive_arp_packet.link_type;
    send_arp_packet.internet_type = receive_arp_packet.internet_type;
    send_arp_packet.src_link_addr = receive_arp_packet.dst_link_addr;
    send_arp_packet.src_internet_addr = receive_arp_packet.dst_internet_addr;
    send_arp_packet.internet_addr_length = receive_arp_packet.internet_addr_length;
    send_arp_packet.link_addr_length = receive_arp_packet.link_addr_length;

    send_arp_packet.operation = op;

    // 自身のアドレスを書き込んで教える
    send_arp_packet.dst_link_addr = opt.dev_addr;
    send_arp_packet.dst_internet_addr = opt.ip_addr;

    ethernet::tx(
        opt,
        dev,
        InternetProtocol::ARP,
        receive_arp_packet.src_link_addr,
        send_arp_packet.to_bytes(InternetProtocolError::CannotConstructPacket)?,
    )?;

    Ok(())
}
