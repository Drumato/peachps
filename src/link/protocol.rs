use thiserror::Error;

use crate::{link, network_device, Items, RxResult};

use super::ethernet;

pub const MTU: usize = 1500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LinkProtocol {
    Ethernet,
}

#[derive(Error, Debug, Clone, Copy)]
pub enum LinkProtocolError {
    #[error("cannot parse frame header")]
    CannotParseFrameHeader,
    #[error("cannot construct frame")]
    CannotConstructFrame,
    #[error("ignore this frame")]
    Ignore,
    #[error("{e:}")]
    NetworkDeviceError {
        e: network_device::NetworkDeviceError,
    },
}

pub async fn rx<'a, ND: network_device::NetworkDevice>(
    items: &'a Items<ND>,
    lp: LinkProtocol,
    buf: &[u8],
) -> Result<(RxResult, Vec<u8>), LinkProtocolError> {
    match lp {
        LinkProtocol::Ethernet => {
            let (frame_header, rest) = ethernet::rx(items, buf).await?;

            let mut result = RxResult::default();
            result.src_mac_addr = frame_header.src_addr;
            result.ip_type = frame_header.ty;

            Ok((result, rest))
        }
    }
}

impl std::fmt::Display for LinkProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            LinkProtocol::Ethernet => "Ethernet",
        };
        write!(f, "{}", s)
    }
}

impl Default for LinkProtocol {
    fn default() -> Self {
        LinkProtocol::Ethernet
    }
}

impl From<network_device::NetworkDeviceError> for LinkProtocolError {
    fn from(e: network_device::NetworkDeviceError) -> Self {
        Self::NetworkDeviceError { e }
    }
}
