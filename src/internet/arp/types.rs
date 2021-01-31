use crate::{internet::ip::IPv4Addr, link::LinkProtocol};
use crate::{internet::InternetProtocol, link::MacAddress};

pub struct ARPHeader {
    pub link_type: LinkProtocol,
    pub internet_type: InternetProtocol,
    pub link_addr_length: u8,
    pub internet_addr_length: u8,
    pub operation: Operation,
    pub src_link_addr: MacAddress,
    pub src_internet_addr: IPv4Addr,
    pub dst_link_addr: MacAddress,
    pub dst_internet_addr: IPv4Addr,
}

impl std::fmt::Display for ARPHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Hardware Address Space: {}", self.link_type)?;
        writeln!(f, "Protocol Address Space: {}", self.internet_type)?;
        writeln!(f, "Hardware Address Length: {}", self.link_addr_length)?;
        writeln!(f, "Protocol Address Length: {}", self.internet_addr_length)?;
        writeln!(f, "Operation: {}", self.operation)?;
        writeln!(f, "Source Hardware Address: {}", self.src_link_addr)?;
        writeln!(f, "Source Protocol Address: {}", self.src_internet_addr)?;
        writeln!(f, "Destination Hardware Address: {}", self.dst_link_addr)?;
        writeln!(
            f,
            "Destination Protocol Address: {}",
            self.dst_internet_addr
        )
    }
}

pub enum Operation {
    Reply,
    Request,
}

impl Default for ARPHeader {
    fn default() -> Self {
        Self {
            link_type: Default::default(),
            internet_type: Default::default(),
            link_addr_length: 0,
            internet_addr_length: 0,
            operation: Operation::Request,
            src_link_addr: Default::default(),
            src_internet_addr: Default::default(),
            dst_link_addr: Default::default(),
            dst_internet_addr: Default::default(),
        }
    }
}
impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Operation::Reply => "ARP Reply",
            Operation::Request => "ARP Request",
        };
        write!(f, "{}", s)
    }
}
impl From<u16> for Operation {
    fn from(v: u16) -> Self {
        match v {
            1 => Operation::Request,
            2 => Operation::Reply,
            _ => unreachable!(),
        }
    }
}

impl Into<u16> for Operation {
    fn into(self) -> u16 {
        match self {
            Operation::Request => 1,
            Operation::Reply => 2,
        }
    }
}
