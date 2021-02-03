use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use internet::InternetProtocolError;
use link::MacAddress;
use transport::TransportProtocol;

use super::{IPHeader, IPv4Addr, IP_BROADCAST_ADDRESS};
use crate::{
    checksum,
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

pub fn rx<'a>(
    opt: Arc<option::PeachPSOption>,
    mut rx_result: RxResult,
    buf: &'a [u8],
) -> Result<(RxResult, Vec<u8>), InternetProtocolError> {
    let ip_packet_hdr =
        IPHeader::new_from_bytes(buf, InternetProtocolError::CannotParsePacketHeader)?;

    if ip_packet_hdr.ihl_bytes_from_vhl() > IPHeader::LEAST_LENGTH {
        return Err(InternetProtocolError::UnsupportedHeaderOption);
    }

    rx_result.src_ip_addr = ip_packet_hdr.src_addr;
    rx_result.tp_type = ip_packet_hdr.protocol;
    let (_, rest) = buf.split_at(ip_packet_hdr.ihl_bytes_from_vhl() as usize);

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
    opt: Arc<option::PeachPSOption>,
    dev: Arc<Mutex<ND>>,
    tp: TransportProtocol,
    rx_result: RxResult,
    tp_payload: Vec<u8>,
    arp_cache: Arc<Mutex<HashMap<internet::ip::IPv4Addr, link::MacAddress>>>,
) -> Result<(), InternetProtocolError> {
    let next_hop = if rx_result.src_ip_addr == internet::ip::IP_BROADCAST_ADDRESS {
        None
    } else {
        // TODO: Find route
        // TODO: use route to determine next hop
        Some(rx_result.src_ip_addr)
    };
    // TODO: segmentation
    tx_core(next_hop, opt, dev, rx_result, tp, tp_payload, arp_cache)?;

    Ok(())
}

fn tx_core<ND: network_device::NetworkDevice>(
    _next_hop: Option<IPv4Addr>,
    opt: Arc<option::PeachPSOption>,
    dev: Arc<Mutex<ND>>,
    rx_result: RxResult,
    tp: TransportProtocol,
    mut tp_payload: Vec<u8>,
    arp_cache: Arc<Mutex<HashMap<internet::ip::IPv4Addr, link::MacAddress>>>,
) -> Result<(), InternetProtocolError> {
    let mut ip_packet = Vec::<u8>::new();

    let dst_ip = rx_result.src_ip_addr;
    let mut packet_hdr = IPHeader {
        version_ihl: IPHeader::VERSION4.checked_shl(4).unwrap()
            | IPHeader::LEAST_LENGTH.checked_shr(2).unwrap() as u8,
        type_of_service: 0,
        total_length: (IPHeader::LEAST_LENGTH as usize + tp_payload.len()) as u16,
        identification: rand::random::<u16>(),
        flg_offset: 0x0,
        time_to_live: 0xff,
        protocol: tp,
        checksum: 0,
        src_addr: opt.ip_addr,
        dst_addr: dst_ip,
    };

    let raw_packet_hdr = packet_hdr.to_bytes(InternetProtocolError::CannotConstructPacket)?;
    packet_hdr.checksum = checksum::calculate_checksum_u16(
        &raw_packet_hdr,
        IPHeader::LEAST_LENGTH as u16,
        InternetProtocolError::CannotConstructPacket,
    )?;

    if opt.debug {
        eprintln!("++++++++ tx ip packet ++++++++");
        eprintln!("{}", packet_hdr);
    }

    ip_packet.append(&mut packet_hdr.to_bytes(InternetProtocolError::CannotConstructPacket)?);
    ip_packet.append(&mut tp_payload);

    if let Ok(cache) = arp_cache.lock() {
        if let Some(a) = cache.get(&rx_result.src_ip_addr) {
            link::ethernet::tx(opt, dev, InternetProtocol::IP, *a, ip_packet)?;
            return Ok(());
        }
    }

    internet::arp::tx_request(Arc::clone(&opt), Arc::clone(&dev), dst_ip)?;

    let dst_mac_addr: Arc<Mutex<Option<MacAddress>>> = Arc::new(Mutex::new(None));
    let dst1 = Arc::clone(&dst_mac_addr);

    std::thread::spawn(move || {
        for _ in 0..5 {
            if let Ok(cache) = arp_cache.lock() {
                if let Some(a) = cache.get(&dst_ip) {
                    *dst1.lock().unwrap() = Some(*a);
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    })
    .join()
    .unwrap();

    if dst_mac_addr.lock().unwrap().is_none() {
        return Err(InternetProtocolError::CannotResolveMACAddressFrom { unknown_ip: dst_ip });
    }

    link::ethernet::tx(
        opt,
        dev,
        InternetProtocol::IP,
        dst_mac_addr.lock().unwrap().unwrap(),
        ip_packet,
    )?;

    Ok(())
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
    if raw_packet_len < (packet_hdr.ihl_bytes_from_vhl().into())
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
