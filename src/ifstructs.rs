use nix::{ioctl_write_int, ioctl_write_ptr_bad, libc};

pub const TUNTAP_MAGIC: u8 = b'T';
pub const TUNSETIFF: u8 = 202;
pub const IFF_TAP: nix::libc::c_short = 0x0002;
pub const IFF_NO_PI: nix::libc::c_short = 0x1000;

// Using ioctl_write_ptr! did not work. Somehow, the kernel expects
// an "int" as input.
ioctl_write_int!(tun_set_iff, TUNTAP_MAGIC, TUNSETIFF);
ioctl_write_ptr_bad!(tun_hardware_address, nix::libc::SIOCGIFHWADDR, IfReq);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IfReq {
    name: [u8; nix::libc::IF_NAMESIZE],
    pub data: IfReqUnion,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct IfMap {
    mem_start: libc::c_ulong,
    mem_end: libc::c_ulong,
    base_addr: libc::c_uchar,
    irq: libc::c_uchar,
    dma: libc::c_uchar,
    port: libc::c_uchar,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union IfReqUnion {
    pub ifr_addr: libc::sockaddr,
    pub ifr_dstaddr: libc::sockaddr,
    pub ifr_broadaddr: libc::sockaddr,
    pub ifr_netmask: libc::sockaddr,
    pub ifr_hwaddr: libc::sockaddr,
    pub ifr_flags: libc::c_short,
    pub ifr_ifindex: libc::c_int,
    pub ifr_metric: libc::c_int,
    pub ifr_mtu: libc::c_int,
    pub ifr_map: IfMap,
    pub ifr_slave: [libc::c_uchar; 16],
    pub ifr_newname: [libc::c_uchar; 16],
    pub ifr_data: *const libc::c_uchar,
}

impl IfReq {
    pub fn new(name: &str) -> Self {
        let mut ifreq = Self {
            name: [0; nix::libc::IF_NAMESIZE],
            data: IfReqUnion { ifr_flags: 0 },
        };

        for (i, byte) in name.as_bytes().iter().enumerate() {
            ifreq.name[i] = *byte;
        }
        ifreq
    }
}
