use crate::{internet, link};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PeachPSOption {
    pub dev_addr: link::MacAddress,
    pub ip_addr: internet::ip::IPv4Addr,
    pub network_mask: internet::ip::IPv4Addr,
    pub debug: bool,
}

impl Default for PeachPSOption {
    fn default() -> Self {
        Self {
            dev_addr: Default::default(),
            ip_addr: Default::default(),
            network_mask: Default::default(),
            debug: false,
        }
    }
}
