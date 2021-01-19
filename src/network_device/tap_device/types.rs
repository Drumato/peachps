use crate::network_device;
use crate::{link, network_device::NetworkDeviceError};
use async_trait::async_trait;

#[repr(C)]
#[derive(Debug)]
pub struct RawTapDevice {
    pub fd: network_device::FileDescriptor,
    pub mac_addr: link::RawMacAddress,
}

pub struct TapDevice {
    pub fd: network_device::FileDescriptor,
    pub mac_addr: link::MacAddress,
}

#[async_trait]
impl network_device::NetworkDevice for TapDevice {
    #[allow(dead_code)]
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, NetworkDeviceError> {
        eprintln!("tap device's addr => {}", self.mac_addr);
        let result = unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, 2048) };

        if result == -1 {
            return Err(NetworkDeviceError::FailedToReadFrom { fd: self.fd });
        }

        Ok(result as usize)
    }
    fn device_addr(&self) -> link::MacAddress {
        self.mac_addr
    }
}

impl TapDevice {
    pub unsafe fn from_raw(fd: libc::c_int, mac_addr: link::RawMacAddress) -> Self {
        Self {
            fd,
            mac_addr: link::MacAddress(mac_addr),
        }
    }
}
