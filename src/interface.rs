use nix::{
    fcntl, ioctl_write_int,
    sys::{socket, stat},
};
use thiserror::Error;

use crate::protocol::ether::MacAddress;

const TUNTAP_MAGIC: u8 = b'T';
const TUNSETIFF: u8 = 202;
const IFF_TAP: nix::libc::c_int = 0x0002;
const IFF_NO_PI: nix::libc::c_int = 0x1000;

// Using ioctl_write_ptr! did not work. Somehow, the kernel expects
// an "int" as input.
ioctl_write_int!(tun_set_iff, TUNTAP_MAGIC, TUNSETIFF);

#[derive(Error, Debug)]
pub enum TAPDeviceError {
    #[error("failed to open /dev/net/tun")]
    Open,
    #[error("failed to create a socket")]
    CreateASocket,
}

#[repr(C)]
#[derive(Debug)]
struct IfReq {
    name: [nix::libc::c_char; nix::libc::IF_NAMESIZE],
    data: nix::libc::c_int,
}

pub struct TAPDevice {
    addr: MacAddress,
}

impl TAPDevice {
    const DEVICE_PATH: &'static str = "/dev/net/tun";
    const TUNSETIFF: u8 = 202;

    pub fn create(interface_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let raw_fd = fcntl::open(
            Self::DEVICE_PATH,
            fcntl::OFlag::O_RDWR,
            stat::Mode::from_bits_truncate(0o644),
        )?;
        if raw_fd == -1 {
            return Err(Box::new(TAPDeviceError::Open));
        }

        let mut ifr = IfReq::new(interface_name, IFF_NO_PI | IFF_TAP);
        unsafe {
            tun_set_iff(raw_fd, &mut ifr as *mut IfReq as u64)?;
        }

        let addr = tap_device_address(interface_name);
        Ok(Self {})
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

    let irf = IfReq::new(interface_name, nix::libc::AF_INET);
}

impl IfReq {
    pub fn new(name: &str, data: nix::libc::c_int) -> Self {
        let mut ifreq = Self {
            name: [0; nix::libc::IF_NAMESIZE],
            data,
        };
        for (i, byte) in name.as_bytes().iter().enumerate() {
            ifreq.name[i] = *byte as nix::libc::c_char
        }
        ifreq
    }
}
