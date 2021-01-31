use std::io::Cursor;

use byteorder_wrapper::read_u8;
use internet::InternetProtocolError;
use transport::TransportProtocol;

use super::{IPHeader, IPv4Addr, IP_BROADCAST_ADDRESS};
use crate::{
    byteorder_wrapper, checksum,
    internet::{self, InternetProtocol},
    link, network_device, option, transport, RxResult,
};

/// プロトコルの動作モード
enum ProcessMode {
    /// 自身に向けられたパケットを受理した場合
    Me,
    /// 他のホストに向けられたパケットの場合
    AnotherHost,
}

pub fn rx(
    opt: option::PeachPSOption,
    mut rx_result: RxResult,
    buf: &[u8],
) -> Result<(RxResult, Vec<u8>), InternetProtocolError> {
    let ip_packet_hdr = parse_ip_packet(buf)?;
    if opt.debug {
        eprintln!("++++++++ IP Packet ++++++++");
        eprintln!("{}", ip_packet_hdr);
    }

    if ip_packet_hdr.ihl_bytes_from_vhl() > IPHeader::LEAST_LENGTH {
        return Err(InternetProtocolError::UnsupportedHeaderOption);
    }

    rx_result.src_ip_addr = ip_packet_hdr.src_addr;
    let (_, rest) = buf.split_at(ip_packet_hdr.ihl_bytes_from_vhl());

    let _mode = validate_ip_packet(
        buf,
        &ip_packet_hdr,
        opt.ip_addr,
        opt.network_mask,
        buf.len(),
    )?;

    Ok((rx_result, rest.to_vec()))
}

pub fn tx<ND: network_device::NetworkDevice>(
    opt: option::PeachPSOption,
    dev: &mut ND,
    tp: TransportProtocol,
    rx_result: RxResult,
    tp_payload: Vec<u8>,
) -> Result<(), InternetProtocolError> {
    let next_hop = if rx_result.src_ip_addr == internet::ip::IP_BROADCAST_ADDRESS {
        None
    } else {
        // TODO: Find route
        // TODO: use route to determine next hop
        Some(rx_result.src_ip_addr)
    };
    let id = rand::random::<u16>();

    // TODO: segmentation
    tx_core(next_hop, id, opt, dev, rx_result, tp, tp_payload)?;

    Ok(())
}

fn tx_core<ND: network_device::NetworkDevice>(
    _next_hop: Option<IPv4Addr>,
    id: u16,
    opt: option::PeachPSOption,
    dev: &mut ND,
    rx_result: RxResult,
    tp: TransportProtocol,
    mut tp_payload: Vec<u8>,
) -> Result<(), InternetProtocolError> {
    let mut ip_packet = Vec::<u8>::new();
    let mut packet_hdr = IPHeader {
        version_ihl: IPHeader::VERSION4 << 4 | (IPHeader::LEAST_LENGTH >> 2) as u8,
        type_of_service: 0,
        total_length: (IPHeader::LEAST_LENGTH + tp_payload.len()) as u16,
        identification: id,
        flg_offset: 0x0,
        time_to_live: 0xff,
        protocol: tp,
        checksum: 0,
        src_addr: opt.ip_addr,
        dst_addr: rx_result.src_ip_addr,
    };

    let mut raw_packet_hdr = packet_hdr.to_bytes(InternetProtocolError::CannotConstructIPPacket)?;
    packet_hdr.checksum = checksum::calculate_checksum_u16(
        &raw_packet_hdr,
        packet_hdr.ihl_bytes_from_vhl() as u16,
        InternetProtocolError::CannotConstructIPPacket,
    )?;
    ip_packet.append(&mut raw_packet_hdr);
    ip_packet.append(&mut tp_payload);

    // TODO: resolve ARP if netdev needs
    link::ethernet::tx(
        opt,
        dev,
        InternetProtocol::IP,
        rx_result.src_mac_addr,
        ip_packet,
    )?;

    Ok(())
}

fn parse_ip_packet(buf: &[u8]) -> Result<IPHeader, InternetProtocolError> {
    let mut reader = Cursor::new(buf);
    let mut packet_hdr: IPHeader = Default::default();

    packet_hdr.version_ihl = byteorder_wrapper::read_u8(
        &mut reader,
        internet::InternetProtocolError::CannotParsePacketHeader,
    )?;

    packet_hdr.type_of_service = byteorder_wrapper::read_u8(
        &mut reader,
        internet::InternetProtocolError::CannotParsePacketHeader,
    )?;
    packet_hdr.total_length = byteorder_wrapper::read_u16_as_be(
        &mut reader,
        internet::InternetProtocolError::CannotParsePacketHeader,
    )?;
    packet_hdr.identification = byteorder_wrapper::read_u16_as_be(
        &mut reader,
        internet::InternetProtocolError::CannotParsePacketHeader,
    )?;
    packet_hdr.flg_offset = byteorder_wrapper::read_u16_as_be(
        &mut reader,
        internet::InternetProtocolError::CannotParsePacketHeader,
    )?;
    packet_hdr.time_to_live = byteorder_wrapper::read_u8(
        &mut reader,
        internet::InternetProtocolError::CannotParsePacketHeader,
    )?;
    packet_hdr.protocol = transport::TransportProtocol::from(byteorder_wrapper::read_u8(
        &mut reader,
        internet::InternetProtocolError::CannotParsePacketHeader,
    )?);
    packet_hdr.checksum = byteorder_wrapper::read_u16_as_be(
        &mut reader,
        internet::InternetProtocolError::CannotParsePacketHeader,
    )?;
    packet_hdr.src_addr = IPv4Addr(byteorder_wrapper::read_u32_as_be(
        &mut reader,
        internet::InternetProtocolError::CannotParsePacketHeader,
    )?);
    packet_hdr.dst_addr = IPv4Addr(byteorder_wrapper::read_u32_as_be(
        &mut reader,
        internet::InternetProtocolError::CannotParsePacketHeader,
    )?);

    // オプションは読み飛ばす
    if packet_hdr.ihl_bytes_from_vhl() > IPHeader::LEAST_LENGTH {
        for _ in 0..(packet_hdr.ihl_bytes_from_vhl() - IPHeader::LEAST_LENGTH) {
            let _ = read_u8(
                &mut reader,
                internet::InternetProtocolError::CannotParsePacketHeader,
            )?;
        }
    }

    Ok(packet_hdr)
}

fn validate_ip_packet(
    raw_packet: &[u8],
    packet_hdr: &IPHeader,
    ip_addr: IPv4Addr,
    network_mask: IPv4Addr,
    raw_packet_len: usize,
) -> Result<ProcessMode, internet::InternetProtocolError> {
    if packet_hdr.version_from_vhl() != 4 {
        return Err(internet::InternetProtocolError::NotIPv4Packet);
    }

    // ヘッダに格納されている"IPパケットヘッダ長" もしくは "パケットの全長"が
    // 実際のバッファサイズより大きければエラーとする
    if raw_packet_len < (packet_hdr.ihl_bytes_from_vhl())
        || raw_packet_len < packet_hdr.total_length as usize
    {
        return Err(internet::InternetProtocolError::InvalidPacketLength);
    }

    if checksum::calculate_checksum_u16(
        raw_packet,
        packet_hdr.ihl_bytes_from_vhl() as u16,
        internet::InternetProtocolError::InvalidChecksum,
    )? != 0
    {
        return Err(internet::InternetProtocolError::InvalidChecksum);
    }

    if packet_hdr.time_to_live == 0 {
        return Err(internet::InternetProtocolError::PacketWasDead);
    }

    // ホストに向けられたパケットであればOK
    if ip_addr == packet_hdr.dst_addr {
        return Ok(ProcessMode::Me);
    }

    // ブロードキャストパケットであるかのチェック
    if ip_addr == ip_addr.to_broadcast(network_mask) || ip_addr == IP_BROADCAST_ADDRESS {
        return Ok(ProcessMode::Me);
    }

    Ok(ProcessMode::AnotherHost)
}

#[cfg(test)]
mod protocol_tests {
    // use super::*;
}
