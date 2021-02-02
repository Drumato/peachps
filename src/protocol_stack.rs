use std::collections::{HashMap, HashSet};

use crate::network_device;
use crate::{internet, link, option, transport};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PeachPSError {
    #[error("got EOF")]
    EOF,
    #[error("should not process this data")]
    ShouldNotProcess,
    #[error("network device error: {e:?}")]
    NetworkDeviceError {
        e: network_device::NetworkDeviceError,
    },
    #[error("link protocol error: {e:?}")]
    LinkProtocolError { e: link::LinkProtocolError },
    #[error("internet protocol error: {e:?}")]
    InternetProtocolError { e: internet::InternetProtocolError },
    #[error("transport protocol error: {e:?}")]
    TransportProtocolError {
        e: transport::TransportProtocolError,
    },
    #[error("ignore this data")]
    Ignore,
}

pub struct RxResult {
    pub src_mac_addr: link::MacAddress,
    pub src_ip_addr: internet::ip::IPv4Addr,
    pub ip_type: internet::InternetProtocol,
    pub tp_type: transport::TransportProtocol,
}

fn rx_datalink<ND>(
    opt: option::PeachPSOption,
    dev: &mut ND,
    lp: link::LinkProtocol,
) -> Result<(RxResult, Vec<u8>), PeachPSError>
where
    ND: network_device::NetworkDevice,
{
    let mut buf: [u8; 2048] = [0; 2048];

    let nbytes = dev.read(&mut buf)?;

    if nbytes == 0 {
        return Err(PeachPSError::EOF);
    }

    let (result, rest) = link::rx(opt, lp, &buf)?;

    Ok((result, rest))
}

fn rx_internet<ND>(
    opt: option::PeachPSOption,
    dev: &mut ND,
    lp: link::LinkProtocol,
    ips: &HashSet<internet::InternetProtocol>,
    arp_cache: &mut HashMap<internet::ip::IPv4Addr, link::MacAddress>,
) -> Result<(RxResult, Vec<u8>), PeachPSError>
where
    ND: network_device::NetworkDevice,
{
    let (link_ex_result, raw_ip_packet) = rx_datalink(opt, dev, lp)?;
    let (result, rest) = internet::rx(opt, dev, ips, link_ex_result, &raw_ip_packet, arp_cache)?;

    Ok((result, rest))
}

fn rx_transport<ND: network_device::NetworkDevice>(
    opt: option::PeachPSOption,
    dev: &mut ND,
    lp: link::LinkProtocol,
    ips: &HashSet<internet::InternetProtocol>,
    tps: &HashSet<transport::TransportProtocol>,
    arp_cache: &mut HashMap<internet::ip::IPv4Addr, link::MacAddress>,
) -> Result<Vec<u8>, PeachPSError> {
    let (result, raw_segment) = rx_internet(opt, dev, lp, ips, arp_cache)?;

    let data = transport::rx(opt, dev, tps, result, &raw_segment, arp_cache)?;

    Ok(data)
}

pub async fn run<ND: network_device::NetworkDevice>(
    opt: option::PeachPSOption,
    dev: &mut ND,
    lp: link::LinkProtocol,
    ips: &HashSet<internet::InternetProtocol>,
    tps: &HashSet<transport::TransportProtocol>,
) -> Result<(), PeachPSError> {
    if opt.debug {
        eprintln!("IP Address: {}", opt.ip_addr);
        eprintln!("Network Mask: {}", opt.network_mask);
    }

    let mut arp_cache: HashMap<internet::ip::IPv4Addr, link::MacAddress> =
        HashMap::with_capacity(16);

    loop {
        match rx_transport(opt, dev, lp, ips, tps, &mut arp_cache) {
            Ok(_data) => {}
            Err(e) => match e {
                PeachPSError::Ignore => {
                    continue;
                }
                _ => return Err(e),
            },
        }
    }
}

impl From<network_device::NetworkDeviceError> for PeachPSError {
    fn from(e: network_device::NetworkDeviceError) -> Self {
        Self::NetworkDeviceError { e }
    }
}

impl From<transport::TransportProtocolError> for PeachPSError {
    fn from(e: transport::TransportProtocolError) -> Self {
        match e {
            transport::TransportProtocolError::Ignore => PeachPSError::Ignore,
            _ => Self::TransportProtocolError { e },
        }
    }
}

impl From<internet::InternetProtocolError> for PeachPSError {
    fn from(e: internet::InternetProtocolError) -> Self {
        match e {
            internet::InternetProtocolError::Ignore => PeachPSError::Ignore,
            _ => Self::InternetProtocolError { e },
        }
    }
}
impl From<link::LinkProtocolError> for PeachPSError {
    fn from(e: link::LinkProtocolError) -> Self {
        match e {
            link::LinkProtocolError::Ignore => PeachPSError::Ignore,
            _ => Self::LinkProtocolError { e },
        }
    }
}

impl Default for RxResult {
    fn default() -> Self {
        Self {
            src_mac_addr: Default::default(),
            src_ip_addr: Default::default(),
            ip_type: Default::default(),
            tp_type: Default::default(),
        }
    }
}
