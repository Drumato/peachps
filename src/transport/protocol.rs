use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    internet::{self, InternetProtocolError},
    link, network_device, option, RxResult,
};

use super::icmp;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum TransportProtocol {
    ICMP,
    TCP,
    UDP,
    UnAssigned,
}

pub trait TransportHeader {}

#[derive(thiserror::Error, Debug, Clone, Copy)]
pub enum TransportProtocolError {
    #[error("cannot parse ICMP message")]
    CannotParseICMPMessage,
    #[error("ignore this segment")]
    Ignore,
    #[error("cannot construct ICMP message")]
    CannotConstructICMPMessage,
    #[error("invalid checksum")]
    InvalidChecksum,
    #[error("{e:}")]
    IPError { e: InternetProtocolError },
}

pub fn rx<ND: 'static + network_device::NetworkDevice>(
    opt: Arc<option::PeachPSOption>,
    dev: Arc<Mutex<ND>>,
    ip_result: RxResult,
    buf: &[u8],
    arp_cache: Arc<Mutex<HashMap<internet::ip::IPv4Addr, link::MacAddress>>>,
) -> Result<Vec<u8>, TransportProtocolError> {
    if !opt.transport_filter.contains(&ip_result.tp_type) {
        return Err(TransportProtocolError::Ignore);
    }

    match ip_result.tp_type {
        TransportProtocol::ICMP => {
            let (_message_header, rest) = icmp::rx(opt, dev, ip_result, buf, arp_cache)?;
            Ok(rest)
        }
        TransportProtocol::TCP => {
            // let (_segment_header, rest) = tcp::rx(opt, dev, ip_result, buf, arp_cache)?;
            // Ok(rest)
            unimplemented!()
        }
        _ => unimplemented!(),
    }
}

impl std::fmt::Display for TransportProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_str = match self {
            TransportProtocol::ICMP => "ICMP",
            TransportProtocol::TCP => "TCP",
            TransportProtocol::UDP => "UDP",
            TransportProtocol::UnAssigned => "UnAssigned",
        };
        write!(f, "{}", type_str)
    }
}
impl From<&str> for TransportProtocol {
    fn from(s: &str) -> Self {
        match s {
            "ICMP" => TransportProtocol::ICMP,
            "TCP" => TransportProtocol::TCP,
            "UDP" => TransportProtocol::UDP,
            _ => panic!("unsupported protocol => '{}'", s),
        }
    }
}

impl From<u8> for TransportProtocol {
    fn from(v: u8) -> Self {
        match v {
            1 => TransportProtocol::ICMP,
            6 => TransportProtocol::TCP,
            17 => TransportProtocol::UDP,
            2 => TransportProtocol::UnAssigned,
            21..=63 => TransportProtocol::UnAssigned,
            _ => panic!("unsupported transport protocol => {}", v),
        }
    }
}

impl Into<u8> for TransportProtocol {
    fn into(self) -> u8 {
        match self {
            TransportProtocol::ICMP => 1,
            TransportProtocol::TCP => 6,
            TransportProtocol::UDP => 17,
            TransportProtocol::UnAssigned => panic!("now allowed into() with unassigned protocol"),
        }
    }
}

impl Default for TransportProtocol {
    fn default() -> Self {
        Self::TCP
    }
}
