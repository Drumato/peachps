use crate::{internet::InternetProtocolError, network_device, Items, RxResult};

use super::{icmp, tcp};

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
    #[error("cannot parse TCP segment")]
    CannotParseTCPSegment,
    #[error("ignore this segment")]
    Ignore,
    #[error("cannot construct ICMP message")]
    CannotConstructICMPMessage,
    #[error("invalid checksum")]
    InvalidChecksum,
    #[error("{e:}")]
    IPError { e: InternetProtocolError },
}

pub async fn rx<'a, ND: network_device::NetworkDevice>(
    table: &'a Items<ND>,
    ip_result: RxResult,
    buf: &[u8],
) -> Result<Vec<u8>, TransportProtocolError> {
    if !table.opt.transport_filter.contains(&ip_result.tp_type) {
        return Err(TransportProtocolError::Ignore);
    }

    match ip_result.tp_type {
        TransportProtocol::ICMP => {
            let (_message_header, rest) = icmp::rx(table, ip_result, buf).await?;
            Ok(rest)
        }
        TransportProtocol::TCP => {
            let (_segment_header, rest) = tcp::rx(table, ip_result, buf).await?;
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

impl From<InternetProtocolError> for TransportProtocolError {
    fn from(e: InternetProtocolError) -> Self {
        match e {
            InternetProtocolError::Ignore => Self::Ignore,
            _ => Self::IPError { e },
        }
    }
}

impl Default for TransportProtocol {
    fn default() -> Self {
        Self::TCP
    }
}
