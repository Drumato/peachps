use std::unimplemented;

use crate::link;

#[allow(dead_code)]
/// Ethernet Frame Header
pub struct FrameHeader {
    pub dst_addr: link::MacAddress,
    pub src_addr: link::MacAddress,
    pub ty: FrameType,
}

impl link::Frame for FrameHeader {
    fn dst_addr(&self) -> link::MacAddress {
        self.dst_addr
    }
    fn frame_type(&self) -> link::FrameType {
        self.ty
    }
}
impl std::fmt::Display for FrameHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "dst_addr: {}", self.dst_addr)?;
        writeln!(f, "src_addr: {}", self.src_addr)?;
        writeln!(f, "frame_type: {}", self.ty)?;

        Ok(())
    }
}
impl FrameHeader {
    pub fn size() -> usize {
        0xe
    }
}
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]

pub enum FrameType {
    /// Internet Protocol
    IP,
    /// Address Resolution Protocol
    ARP,
    /// Internet Protocol Version 6
    IPV6,
}

impl std::fmt::Display for FrameType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_str = match self {
            FrameType::IP => "IP",
            FrameType::ARP => "ARP",
            FrameType::IPV6 => "IPv6",
        };
        write!(f, "{}", type_str)
    }
}

impl Default for FrameHeader {
    fn default() -> Self {
        Self {
            dst_addr: link::MacAddress([0; 6]),
            src_addr: link::MacAddress([0; 6]),
            ty: FrameType::IP,
        }
    }
}

impl From<u16> for FrameType {
    fn from(v: u16) -> Self {
        match v {
            0x0800 => FrameType::IP,
            0x0806 => FrameType::ARP,
            0x86dd => FrameType::IPV6,
            _ => unimplemented!(),
        }
    }
}

impl Into<u16> for FrameType {
    fn into(self) -> u16 {
        match self {
            FrameType::IP => 0x0800,
            FrameType::ARP => 0x0806,
            FrameType::IPV6 => 0x86dd,
        }
    }
}
