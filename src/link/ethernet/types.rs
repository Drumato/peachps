use std::io::Cursor;

use link::MacAddress;

use crate::{byteorder_wrapper, internet, link};

#[allow(dead_code)]
/// Ethernet Frame Header
pub struct FrameHeader {
    pub dst_addr: link::MacAddress,
    pub src_addr: link::MacAddress,
    pub ty: internet::InternetProtocol,
}

impl std::fmt::Display for FrameHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "dst_addr: {}", self.dst_addr)?;
        writeln!(f, "src_addr: {}", self.src_addr)?;
        writeln!(f, "frame_type: {}", self.ty)?;

        Ok(())
    }
}

impl Default for FrameHeader {
    fn default() -> Self {
        Self {
            dst_addr: link::MacAddress([0; 6]),
            src_addr: link::MacAddress([0; 6]),
            ty: internet::InternetProtocol::IP,
        }
    }
}

impl FrameHeader {
    pub const LENGTH: usize = 0xe;

    pub fn to_bytes<E>(&self, err: E) -> Result<Vec<u8>, E>
    where
        E: std::error::Error + Copy,
    {
        let mut buf = Vec::new();
        buf.append(&mut self.dst_addr.to_bytes(err)?);
        buf.append(&mut self.src_addr.to_bytes(err)?);
        byteorder_wrapper::write_u16_as_be(&mut buf, self.ty.into(), err)?;

        Ok(buf)
    }

    pub fn new_from_bytes<E>(buf: &[u8], err: E) -> Result<(Self, Vec<u8>), E>
    where
        E: std::error::Error + Copy,
    {
        let (raw_header, rest) = buf.split_at(FrameHeader::LENGTH);
        let mut reader = Cursor::new(raw_header);
        let mut frame_hdr: FrameHeader = Default::default();

        frame_hdr.dst_addr = MacAddress::from_cursor(&mut reader, err)?;

        frame_hdr.src_addr = MacAddress::from_cursor(&mut reader, err)?;

        let frame_type = byteorder_wrapper::read_u16_as_be(&mut reader, err)?;

        frame_hdr.ty = internet::InternetProtocol::from(frame_type);
        Ok((frame_hdr, rest.to_vec()))
    }
}

#[cfg(test)]
mod tests {
    use link::LinkProtocolError;

    use super::*;

    #[test]
    fn parse_ethernet_frame_test1() {
        let raw_frame = [
            0x00, 0x15, 0x5d, 0x22, 0x1e, 0xff, 0x00, 0x15, 0x5d, 0x74, 0x4d, 0x66, 0x08, 0x00,
        ];
        let result =
            FrameHeader::new_from_bytes(&raw_frame, LinkProtocolError::CannotParseFrameHeader);
        assert!(result.is_ok());
        let frame_hdr = result.unwrap();

        assert_eq!([0x00, 0x15, 0x5d, 0x22, 0x1e, 0xff], frame_hdr.dst_addr.0);
        assert_eq!([0x00, 0x15, 0x5d, 0x74, 0x4d, 0x66], frame_hdr.src_addr.0);
        assert_eq!(internet::InternetProtocol::IP, frame_hdr.ty);
    }

    #[test]
    fn parse_ethernet_frame_test2() {
        let raw_frame = [
            0xa8, 0x5e, 0x45, 0x2f, 0x94, 0x2e, 0x18, 0xec, 0xe7, 0x56, 0x5e, 0x60, 0x08, 0x06,
        ];
        let result =
            FrameHeader::new_from_bytes(&raw_frame, LinkProtocolError::CannotParseFrameHeader);
        assert!(result.is_ok());
        let frame_hdr = result.unwrap();

        assert_eq!([0xa8, 0x5e, 0x45, 0x2f, 0x94, 0x2e], frame_hdr.dst_addr.0);
        assert_eq!([0x18, 0xec, 0xe7, 0x56, 0x5e, 0x60], frame_hdr.src_addr.0);
        assert_eq!(internet::InternetProtocol::ARP, frame_hdr.ty);
    }
}
