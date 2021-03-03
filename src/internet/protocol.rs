use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use internet::ip::IPv4Addr;

use crate::{
    internet,
    link::{self, LinkProtocolError},
    network_device, option, RxResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum InternetProtocol {
    IP,
    ARP,
    IPv6,
}
#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum InternetProtocolError {
    #[error("frame type is ipv4 but version isn't 4 in vhl")]
    NotIPv4Packet,
    #[error("invalid packet length was found")]
    InvalidPacketLength,
    #[error("invalid checksum found")]
    InvalidChecksum,
    #[error("packet was dead (TTL=0)")]
    PacketWasDead,
    #[error("cannot parse packet header")]
    CannotParsePacketHeader,
    #[error("ignore this packet")]
    Ignore,
    #[error("cannot construct packet")]
    CannotConstructPacket,
    #[error("{e:}")]
    LinkError { e: LinkProtocolError },
    #[error("unsupported header option")]
    UnsupportedHeaderOption,
    #[error("cannot resolve MAC address from {unknown_ip:?}")]
    CannotResolveMACAddressFrom { unknown_ip: IPv4Addr },
}

pub fn rx<ND: 'static + network_device::NetworkDevice>(
    opt: Arc<option::PeachPSOption>,
    dev: Arc<Mutex<ND>>,
    rx_result: RxResult,
    buf: &[u8],
    arp_cache: Arc<Mutex<HashMap<internet::ip::IPv4Addr, link::MacAddress>>>,
) -> Result<(RxResult, Vec<u8>), InternetProtocolError> {
    if !opt.internet_filter.contains(&rx_result.ip_type) {
        return Err(InternetProtocolError::Ignore);
    }

    match rx_result.ip_type {
        InternetProtocol::IP => internet::ip::rx(opt, rx_result, buf),
        InternetProtocol::ARP => internet::arp::rx(opt, dev, rx_result, buf, arp_cache),
        _ => unimplemented!(),
    }
}

impl std::fmt::Display for InternetProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_str = match self {
            InternetProtocol::IP => "IP",
            InternetProtocol::ARP => "ARP",
            InternetProtocol::IPv6 => "IPv6",
        };
        write!(f, "{}", type_str)
    }
}
impl From<&str> for InternetProtocol {
    fn from(s: &str) -> Self {
        match s {
            "IP" => InternetProtocol::IP,
            "ARP" => InternetProtocol::ARP,
            "IPv6" => InternetProtocol::IPv6,
            _ => panic!("unsupported protocol => '{}'", s),
        }
    }
}

impl From<u16> for InternetProtocol {
    fn from(v: u16) -> Self {
        match v {
            0x0800 => InternetProtocol::IP,
            0x0806 => InternetProtocol::ARP,
            0x86dd => InternetProtocol::IPv6,
            _ => unimplemented!(),
        }
    }
}

impl Into<u16> for InternetProtocol {
    fn into(self) -> u16 {
        match self {
            InternetProtocol::IP => 0x0800,
            InternetProtocol::ARP => 0x0806,
            InternetProtocol::IPv6 => 0x86dd,
        }
    }
}
impl Default for InternetProtocol {
    fn default() -> Self {
        Self::IP
    }
}

impl From<LinkProtocolError> for InternetProtocolError {
    fn from(e: LinkProtocolError) -> Self {
        match e {
            LinkProtocolError::Ignore => Self::Ignore,
            _ => Self::LinkError { e },
        }
    }
}
