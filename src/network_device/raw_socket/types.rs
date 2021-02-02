use crate::link;
use crate::network_device;

use network_device::NetworkDeviceError;

#[repr(C)]
#[derive(Debug)]
pub struct RawSocket {
    pub fd: network_device::FileDescriptor,
    pub mac_addr: link::RawMacAddress,
}

#[derive(Clone, Copy)]
pub struct Socket {
    pub fd: network_device::FileDescriptor,
    pub mac_addr: link::MacAddress,
}

impl network_device::NetworkDevice for Socket {
    #[allow(dead_code)]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, NetworkDeviceError> {
        let result = unsafe {
            let mut pollfd = libc::pollfd {
                fd: self.fd,
                events: libc::POLLIN,
                revents: 0,
            };
            let ret = libc::poll(&mut pollfd, 1, 3000);
            if ret == -1 && *libc::__errno_location() != libc::EINTR {
                return Err(NetworkDeviceError::FailedToReadFrom { fd: self.fd });
            } else if ret == 0 {
                return Err(NetworkDeviceError::Timeout);
            }
            libc::read(self.fd, buf.as_mut_ptr() as *mut libc::c_void, buf.len())
        };

        if result == -1 {
            return Err(NetworkDeviceError::FailedToReadFrom { fd: self.fd });
        }

        Ok(result as usize)
    }
    fn write(&mut self, buf: &[u8]) -> Result<usize, NetworkDeviceError> {
        let result =
            unsafe { libc::write(self.fd, buf.as_ptr() as *const libc::c_void, buf.len()) };
        if result == -1 || result != buf.len() as isize {
            return Err(NetworkDeviceError::FailedToWriteTo { fd: self.fd });
        }

        Ok(result as usize)
    }

    fn device_addr(&self) -> link::MacAddress {
        self.mac_addr
    }
}

impl Socket {
    pub unsafe fn from_raw(fd: network_device::FileDescriptor, addr: link::RawMacAddress) -> Self {
        Self {
            fd: fd,
            mac_addr: link::MacAddress(addr),
        }
    }
}
