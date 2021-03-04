use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

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

fn rx_datalink<'a, ND>(
    opt: Arc<option::PeachPSOption>,
    dev: Arc<Mutex<ND>>,
    lp: link::LinkProtocol,
) -> Result<(RxResult, Vec<u8>), PeachPSError>
where
    ND: network_device::NetworkDevice,
{
    let mut buf: [u8; 2048] = [0; 2048];

    let nbytes = dev.lock().unwrap().read(&mut buf)?;

    if nbytes == 0 {
        return Err(PeachPSError::EOF);
    }

    let (result, rest) = link::rx(opt, lp, &buf)?;

    Ok((result, rest))
}

fn rx_internet<ND>(
    opt: Arc<option::PeachPSOption>,
    dev: Arc<Mutex<ND>>,
    lp: link::LinkProtocol,
    arp_cache: Arc<Mutex<HashMap<internet::ip::IPv4Addr, link::MacAddress>>>,
) -> Result<(RxResult, Vec<u8>), PeachPSError>
where
    ND: 'static + network_device::NetworkDevice,
{
    let (link_ex_result, raw_ip_packet) = rx_datalink(Arc::clone(&opt), Arc::clone(&dev), lp)?;
    let (result, rest) = internet::rx(opt, dev, link_ex_result, &raw_ip_packet, arp_cache)?;

    Ok((result, rest))
}

fn rx_transport<ND: 'static + network_device::NetworkDevice>(
    opt: Arc<option::PeachPSOption>,
    dev: Arc<Mutex<ND>>,
    lp: link::LinkProtocol,
    arp_cache: Arc<Mutex<HashMap<internet::ip::IPv4Addr, link::MacAddress>>>,
) -> Result<Vec<u8>, PeachPSError> {
    let (result, raw_segment) = rx_internet(
        Arc::clone(&opt),
        Arc::clone(&dev),
        lp,
        Arc::clone(&arp_cache),
    )?;

    let data = transport::rx(opt, dev, result, &raw_segment, arp_cache)?;

    Ok(data)
}

pub fn run<T, ND>(
    opt: option::PeachPSOption,
    dev: ND,
    lp: link::LinkProtocol,
) -> Result<(), PeachPSError>
where
    ND: 'static + network_device::NetworkDevice,
{
    let dev = Arc::new(Mutex::new(dev));
    let opt = Arc::new(opt);
    let arp_cache = Arc::new(Mutex::new(HashMap::with_capacity(16)));

    let rx_thread = std::thread::spawn(move || loop {
        let dev = Arc::clone(&dev);
        let opt = Arc::clone(&opt);
        let arp_cache = Arc::clone(&arp_cache);
        match rx_transport(opt, dev, lp, arp_cache) {
            Ok(_data) => {}
            Err(e) => match e {
                PeachPSError::Ignore => {
                    continue;
                }
                _ => {
                    eprintln!("Error Found: {}", e);
                    std::process::exit(1);
                }
            },
        }
    });

    rx_thread.join().unwrap();

    Ok(())
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
