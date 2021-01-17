use crate::link;
use crate::network_device;
use async_trait::async_trait;
use network_device::NetworkDeviceError;

#[repr(C)]
#[derive(Debug)]
pub struct RawSocket {
    pub fd: network_device::FileDescriptor,
    pub mac_addr: link::RawMacAddress,
}

pub struct Socket {
    pub fd: network_device::FileDescriptor,
    pub mac_addr: link::MacAddress,
}

#[async_trait]
impl network_device::NetworkDevice for Socket {
    #[allow(dead_code)]
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, NetworkDeviceError> {
        let result = unsafe { libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, 2048) };
        if result == -1 {
            return Err(NetworkDeviceError::FailedToReadFrom { fd: self.fd });
        }

        Ok(result as usize)
    }
}

impl Socket {
    pub unsafe fn from_raw(fd: network_device::FileDescriptor, addr: link::RawMacAddress) -> Self {
        Self {
            fd: fd,
            mac_addr: link::MacAddress { addr: addr },
        }
    }
}
