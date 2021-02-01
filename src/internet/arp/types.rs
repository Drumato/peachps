use std::{io::Cursor, unimplemented};

use crate::{byteorder_wrapper, internet::ip::IPv4Addr, link::LinkProtocol};
use crate::{internet::InternetProtocol, link::MacAddress};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Operation {
    Reply,
    Request,
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

impl ARPHeader {
    pub fn new_from_bytes<E: std::error::Error + Copy>(buf: &[u8], err: E) -> Result<Self, E> {
        let mut reader = Cursor::new(buf);
        let mut packet_hdr: Self = Default::default();

        packet_hdr.link_type =
            Self::new_hw_addr_space(byteorder_wrapper::read_u16_as_be(&mut reader, err)?);
        packet_hdr.internet_type =
            Self::new_protocol_addr_space(byteorder_wrapper::read_u16_as_be(&mut reader, err)?);
        packet_hdr.link_addr_length = byteorder_wrapper::read_u8(&mut reader, err)?;
        packet_hdr.internet_addr_length = byteorder_wrapper::read_u8(&mut reader, err)?;
        packet_hdr.operation =
            Operation::from(byteorder_wrapper::read_u16_as_be(&mut reader, err)?);

        packet_hdr.src_link_addr = MacAddress::from_cursor(&mut reader, err)?;

        packet_hdr.src_internet_addr =
            IPv4Addr(byteorder_wrapper::read_u32_as_be(&mut reader, err)?);

        packet_hdr.dst_link_addr = MacAddress::from_cursor(&mut reader, err)?;

        packet_hdr.dst_internet_addr =
            IPv4Addr(byteorder_wrapper::read_u32_as_be(&mut reader, err)?);

        Ok(packet_hdr)
    }

    pub fn to_bytes<E>(&self, err: E) -> Result<Vec<u8>, E>
    where
        E: std::error::Error + Copy,
    {
        let mut payload = Vec::new();

        byteorder_wrapper::write_u16_as_be(&mut payload, self.hw_addr_space_to_bytes(), err)?;
        byteorder_wrapper::write_u16_as_be(&mut payload, self.protocol_addr_space_to_bytes(), err)?;
        byteorder_wrapper::write_u8(&mut payload, self.link_addr_length, err)?;
        byteorder_wrapper::write_u8(&mut payload, self.internet_addr_length, err)?;
        byteorder_wrapper::write_u16_as_be(&mut payload, self.operation.into(), err)?;
        payload.append(&mut self.src_link_addr.to_bytes(err)?);
        payload.append(&mut self.src_internet_addr.to_bytes(err)?);
        payload.append(&mut self.dst_link_addr.to_bytes(err)?);
        payload.append(&mut self.dst_internet_addr.to_bytes(err)?);
        Ok(payload)
    }

    fn hw_addr_space_to_bytes(&self) -> u16 {
        match self.link_type {
            LinkProtocol::Ethernet => 1,
        }
    }
    fn new_hw_addr_space(v: u16) -> LinkProtocol {
        match v {
            1 => LinkProtocol::Ethernet,
            _ => unimplemented!(),
        }
    }
    fn new_protocol_addr_space(v: u16) -> InternetProtocol {
        match v {
            0x0800 => InternetProtocol::IP,
            _ => unimplemented!(),
        }
    }
    fn protocol_addr_space_to_bytes(&self) -> u16 {
        match self.internet_type {
            InternetProtocol::IP => 0x0800,
            _ => unimplemented!(),
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

#[cfg(test)]
mod tests {

    use crate::internet::InternetProtocolError;

    use super::*;

    #[test]
    fn parse_arp_packet_test1() {
        let raw_packet = [
            0x00, 0x01, 0x08, 0x00, 0x06, 0x04, 0x00, 0x01, 0x18, 0xec, 0xe7, 0x56, 0x5e, 0x60,
            0xc0, 0xa8, 0x0b, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0xa8, 0x0b, 0x03,
        ];
        let result =
            ARPHeader::new_from_bytes(&raw_packet, InternetProtocolError::CannotParsePacketHeader);
        assert!(result.is_ok());
        let packet_hdr = result.unwrap();

        assert_eq!(LinkProtocol::Ethernet, packet_hdr.link_type);
        assert_eq!(InternetProtocol::IP, packet_hdr.internet_type);
        assert_eq!(0x6, packet_hdr.link_addr_length);
        assert_eq!(0x4, packet_hdr.internet_addr_length);
        assert_eq!(Operation::Request, packet_hdr.operation);
        assert_eq!(
            MacAddress([0x18, 0xec, 0xe7, 0x56, 0x5e, 0x60,]),
            packet_hdr.src_link_addr
        );
        assert_eq!(IPv4Addr(0xc0a80b01), packet_hdr.src_internet_addr);
        assert_eq!(
            MacAddress([0x00, 0x00, 0x00, 0x00, 0x00, 0x00]),
            packet_hdr.dst_link_addr
        );
        assert_eq!(IPv4Addr(0xc0a80b03), packet_hdr.dst_internet_addr);
    }
}
