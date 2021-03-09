use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    internet,
    link::{self, MacAddress},
    option, transport,
};
use crate::{
    internet::ip::IPv4Addr,
    network_device::{self, NetworkDevice},
};

use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Items<ND: network_device::NetworkDevice> {
    pub opt: option::PeachPSOption,
    pub dev: Arc<Mutex<ND>>,
    pub arp_table: Arc<Mutex<HashMap<internet::ip::IPv4Addr, link::MacAddress>>>,
}

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

/// 下位層から上位層に向かって伝播させる情報の集約
pub struct RxResult {
    pub src_mac_addr: link::MacAddress,
    pub src_ip_addr: internet::ip::IPv4Addr,
    pub ip_type: internet::InternetProtocol,
    pub tp_type: transport::TransportProtocol,
    pub message_len: usize,
}

async fn rx_datalink<'a, ND>(
    table: &'a Items<ND>,
    lp: link::LinkProtocol,
) -> Result<(RxResult, Vec<u8>), PeachPSError>
where
    ND: network_device::NetworkDevice,
{
    let mut buf: [u8; 2048] = [0; 2048];

    if let Ok(dev) = table.dev.lock() {
        let nbytes = dev.read(&mut buf).await?;
        if nbytes == 0 {
            return Err(PeachPSError::EOF);
        }

        let (result, rest) = link::rx(table, lp, &buf).await?;

        return Ok((result, rest));
    }

    unreachable!()
}

async fn rx_internet<'a, ND>(
    table: &'a Items<ND>,
    lp: link::LinkProtocol,
) -> Result<(RxResult, Vec<u8>), PeachPSError>
where
    ND: network_device::NetworkDevice,
{
    let (link_ex_result, raw_ip_packet) = rx_datalink(table, lp).await?;
    let (result, rest) = internet::rx(table, link_ex_result, &raw_ip_packet).await?;

    Ok((result, rest))
}

async fn rx_transport<'a, ND: network_device::NetworkDevice>(
    table: &'a Items<ND>,
    lp: link::LinkProtocol,
) -> Result<Vec<u8>, PeachPSError> {
    let (result, raw_segment) = rx_internet(table, lp).await?;

    let data = transport::rx(table, result, &raw_segment).await?;

    Ok(data)
}

pub async fn run<'a, ND>(table: &'a Items<ND>, lp: link::LinkProtocol) -> Result<(), PeachPSError>
where
    ND: network_device::NetworkDevice,
{
    loop {
        match rx_transport(&table, lp).await {
            Ok(_data) => {}
            Err(e) => match e {
                PeachPSError::Ignore => {}
                _ => {
                    eprintln!("Error Found: {}", e);
                    std::process::exit(1);
                }
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
            message_len: 0,
        }
    }
}

impl<ND: NetworkDevice> Items<ND> {
    pub fn new(opt: option::PeachPSOption, dev: ND) -> Self {
        Self {
            opt,
            dev: Arc::new(Mutex::new(dev)),
            arp_table: Arc::new(Mutex::new(HashMap::with_capacity(16))),
        }
    }

    pub fn lookup_arp_table(&self, ip: &IPv4Addr) -> Option<MacAddress> {
        if let Ok(arp_table) = self.arp_table.lock() {
            if let Some(mac_addr) = arp_table.get(ip) {
                return Some(*mac_addr);
            }
        }

        None
    }
}
