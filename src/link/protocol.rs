use thiserror::Error;

use crate::{link, network_device, option, RxResult};

use super::ethernet;

pub const MTU: usize = 1500;

#[derive(Debug, Clone, Copy)]
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

pub fn rx(
    opt: option::PeachPSOption,
    lp: LinkProtocol,
    buf: &[u8],
) -> Result<(RxResult, Vec<u8>), LinkProtocolError> {
    if buf.len() < link::ethernet::FrameHeader::LENGTH {
        return Err(LinkProtocolError::CannotParseFrameHeader);
    }

    match lp {
        LinkProtocol::Ethernet => {
            let (frame_header, rest) = ethernet::rx(buf)?;
            if opt.debug {
                eprintln!("++++++++ Ethernet Frame ++++++++");
                eprintln!("{}", frame_header);
            }

            if !should_process(opt.dev_addr, frame_header.dst_addr) {
                return Err(LinkProtocolError::Ignore);
            }

            let result = RxResult {
                src_mac_addr: frame_header.src_addr,

                ..Default::default()
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
