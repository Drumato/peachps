use crate::{
    internet::{ip, InternetProtocol, InternetProtocolError},
    link::{self, ethernet, LinkProtocol},
    network_device, Items, RxResult,
};

use super::{ARPHeader, Operation};

pub async fn resolve_mac_address<'a, ND: network_device::NetworkDevice>(
    table: &'a Items<ND>,
    dst_ip: ip::IPv4Addr,
) -> Result<link::MacAddress, InternetProtocolError> {
    tx_request(table, dst_ip).await?;

    for _ in 0..5 {
        if let Ok(arp_table) = table.arp_table.lock() {
            if let Some(dst_mac_addr) = arp_table.get(&dst_ip) {
                return Ok(*dst_mac_addr);
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    Err(InternetProtocolError::CannotResolveMACAddressFrom { unknown_ip: dst_ip })
}

pub async fn rx<'a, ND: network_device::NetworkDevice>(
    table: &'a Items<ND>,
    mut rx_result: RxResult,
    buf: &[u8],
) -> Result<(RxResult, Vec<u8>), InternetProtocolError> {
    let arp_packet_hdr =
        ARPHeader::new_from_bytes(buf, InternetProtocolError::CannotParsePacketHeader)?;
    if table.opt.debug {
        eprintln!("++++++++ tx arp packet ++++++++");
        eprintln!("{}", arp_packet_hdr);
    }

    let (_, rest) = buf.split_at(ARPHeader::LENGTH);

    if arp_packet_hdr.operation == Operation::Request {
        // ARPテーブルのロックをとって書き込む
        if let Ok(ref mut arp_table) = table.arp_table.lock() {
            arp_table.insert(
                arp_packet_hdr.src_internet_addr,
                arp_packet_hdr.src_link_addr,
            );
        }

        tx_reply(table, &arp_packet_hdr).await?;
    }

    rx_result.src_ip_addr = arp_packet_hdr.src_internet_addr;

    Ok((rx_result, rest.to_vec()))
}

pub async fn tx_request<'a, ND: network_device::NetworkDevice>(
    table: &'a Items<ND>,
    target_ip: ip::IPv4Addr,
) -> Result<(), InternetProtocolError> {
    let mut send_arp_packet: ARPHeader = Default::default();

    send_arp_packet.operation = Operation::Request;
    send_arp_packet.src_internet_addr = table.opt.ip_addr;
    send_arp_packet.src_link_addr = table.opt.dev_addr;
    send_arp_packet.dst_internet_addr = target_ip;
    send_arp_packet.link_type = LinkProtocol::Ethernet;
    send_arp_packet.internet_type = InternetProtocol::IP;
    send_arp_packet.internet_addr_length = 6;
    send_arp_packet.link_addr_length = 4;

    ethernet::tx(
        table,
        InternetProtocol::ARP,
        link::MacAddress::BLOADCAST,
        send_arp_packet.to_bytes(InternetProtocolError::CannotConstructPacket)?,
    )
    .await?;

    Ok(())
}

async fn tx_reply<'a, ND: network_device::NetworkDevice>(
    table: &'a Items<ND>,
    receive_arp_packet: &ARPHeader,
) -> Result<(), InternetProtocolError> {
    tx(table, Operation::Reply, receive_arp_packet).await
}

async fn tx<'a, ND: network_device::NetworkDevice>(
    table: &'a Items<ND>,
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
    send_arp_packet.src_link_addr = table.opt.dev_addr;
    send_arp_packet.src_internet_addr = table.opt.ip_addr;

    ethernet::tx(
        table,
        InternetProtocol::ARP,
        send_arp_packet.dst_link_addr,
        send_arp_packet.to_bytes(InternetProtocolError::CannotConstructPacket)?,
    )
    .await?;

    Ok(())
}
