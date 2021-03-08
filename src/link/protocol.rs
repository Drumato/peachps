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
    table: &'a Items<ND>,
    lp: LinkProtocol,
    buf: &[u8],
) -> Result<(RxResult, Vec<u8>), LinkProtocolError> {
    if buf.len() < link::ethernet::FrameHeader::LENGTH {
        return Err(LinkProtocolError::CannotParseFrameHeader);
    }

    match lp {
        LinkProtocol::Ethernet => {
            let (frame_header, rest) = ethernet::rx(buf).await?;

            if !should_process(table.opt.dev_addr, frame_header.dst_addr) {
                return Err(LinkProtocolError::Ignore);
            }

            let result = RxResult {
                src_mac_addr: frame_header.src_addr,
                src_ip_addr: Default::default(),
                ip_type: frame_header.ty,
                tp_type: Default::default(),
            };
            Ok((result, rest))
        }
    }
}

// プロトコルスタックが処理すべきデータかどうか検査
fn should_process(device_addr: link::MacAddress, frame_dst_addr: link::MacAddress) -> bool {
    frame_target_is_nic(device_addr, frame_dst_addr)
        || frame_dst_addr == link::BLOADCAST_MAC_ADDRESS
}
fn frame_target_is_nic(device_addr: link::MacAddress, frame_dst_addr: link::MacAddress) -> bool {
    device_addr == frame_dst_addr
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
