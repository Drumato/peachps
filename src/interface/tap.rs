use nix::{
    fcntl,
    sys::{socket, stat},
    unistd,
};
use thiserror::Error;

use crate::{ifstructs, protocol::ether::MacAddress};

#[derive(Error, Debug)]
pub enum TAPDeviceError {
    #[error("failed to open /dev/net/tun")]
    Open,
    #[error("failed to create a socket")]
    CreateASocket,
    #[error("failed to read")]
    Read,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TAPDevice {
    fd: i32,
    pub addr: MacAddress,
}

impl TAPDevice {
    const DEVICE_PATH: &'static str = "/dev/net/tun";

    pub fn read(&self, buf: &mut [u8]) -> Result<usize, TAPDeviceError> {
        unistd::read(self.fd, buf).map_err(|_e| TAPDeviceError::Read)
    }

    pub fn create(interface_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let raw_fd = fcntl::open(
            Self::DEVICE_PATH,
            fcntl::OFlag::O_RDWR,
            stat::Mode::from_bits_truncate(0o644),
        )?;

        if raw_fd == -1 {
            return Err(Box::new(TAPDeviceError::Open));
        }

        let mut ifr = ifstructs::IfReq::new(interface_name);
        ifr.data.ifr_flags = ifstructs::IFF_TAP | ifstructs::IFF_NO_PI;

        unsafe {
            ifstructs::tun_set_iff(raw_fd, &mut ifr as *mut ifstructs::IfReq as u64)?;
        }

        let addr = tap_device_address(interface_name)?;
        Ok(Self { fd: raw_fd, addr })
    }
}

fn tap_device_address(interface_name: &str) -> Result<MacAddress, Box<dyn std::error::Error>> {
    let sock = socket::socket(
        socket::AddressFamily::Inet,
        socket::SockType::Datagram,
        socket::SockFlag::empty(),
        None,
    )?;
    if sock == -1 {
        return Err(Box::new(TAPDeviceError::CreateASocket));
    }

    let mut ifr = ifstructs::IfReq::new(interface_name);
    unsafe {
        ifstructs::tun_hardware_address(sock, &mut ifr as *mut ifstructs::IfReq)?;
    }

    Ok(MacAddress::from(unsafe { ifr.data.ifr_hwaddr.sa_data }))
}
