use crate::link;
use thiserror::Error;

#[repr(C)]
pub struct RawTapDevice {
    pub fd: libc::c_int,
    pub mac_addr: link::RawMacAddress,
}

pub struct TapDevice {
    pub fd: u32,
    pub mac_addr: link::MacAddress,
}

impl TapDevice {
    pub fn from_raw(fd: libc::c_int, mac_addr: link::RawMacAddress) -> Self {
        Self {
            fd: fd as u32,
            mac_addr: link::MacAddress { addr: mac_addr },
        }
    }
}

#[derive(Error, Debug)]
pub enum TapDeviceError {
    #[error("malformed device name")]
    MalformedDeviceName,
    #[error("malformed interface name")]
    MalformedInterfaceName,
    #[error("failed to setup tap device")]
    FailedToSetupTapDevice,
}
