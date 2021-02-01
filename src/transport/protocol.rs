use std::collections::HashSet;

use crate::{internet::InternetProtocolError, network_device, option, RxResult};

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
    #[error("{e:}")]
    IPError { e: InternetProtocolError },
}

pub fn rx<ND: network_device::NetworkDevice>(
    opt: option::PeachPSOption,
    dev: &mut ND,
    expects: &HashSet<TransportProtocol>,
    ip_result: RxResult,
    buf: &[u8],
) -> Result<Vec<u8>, TransportProtocolError> {
    if !expects.contains(&ip_result.tp_type) {
        return Err(TransportProtocolError::Ignore);
    }
    match ip_result.tp_type {
        TransportProtocol::ICMP => {
            let (_message_header, rest) = icmp::rx(opt, dev, ip_result, buf)?;
            Ok(rest)
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
