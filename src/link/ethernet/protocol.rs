use crate::link;

use byteorder::{BigEndian, ReadBytesExt};

use std::io::Cursor;

/// Ethernetプロトコルの構造体
pub struct Ethernet();

impl link::LinkLayer for Ethernet {
    type FrameHeader = link::FrameHeader;
    /// Ethernetプロトコルの実装
    fn run(&self, buf: &[u8]) -> Result<(link::FrameHeader, Vec<u8>), Box<dyn std::error::Error>> {
        let frame_hdr = self.parse_frame(&buf)?;
        Ok((frame_hdr, buf[link::FrameHeader::size()..].to_vec()))
    }
}

impl Ethernet {
    /// Ethernet Frameのパース
    fn parse_frame(&self, buf: &[u8]) -> Result<link::FrameHeader, Box<dyn std::error::Error>> {
        let mut reader = Cursor::new(buf);
        let mut frame_hdr: link::FrameHeader = Default::default();

        for i in 0..6 {
            frame_hdr.dst_addr.0[i] = reader.read_u8()?;
        }
        for i in 0..6 {
            frame_hdr.src_addr.0[i] = reader.read_u8()?;
        }

        frame_hdr.ty = link::FrameType::from(reader.read_u16::<BigEndian>()?);

        Ok(frame_hdr)
    }
}

#[cfg(test)]
mod protocol_tests {
    use super::*;

    #[test]
    fn parse_ethernet_frame_test() {
        let raw_frame = [
            0x00, 0x15, 0x5d, 0x22, 0x1e, 0xff, 0x00, 0x15, 0x5d, 0x74, 0x4d, 0x66, 0x08, 0x00,
        ];
        let prot = Ethernet();
        let result = prot.parse_frame(&raw_frame);
        assert!(result.is_ok());
        let frame_hdr = result.unwrap();

        assert_eq!([0x00, 0x15, 0x5d, 0x22, 0x1e, 0xff], frame_hdr.dst_addr.0);
        assert_eq!([0x00, 0x15, 0x5d, 0x74, 0x4d, 0x66], frame_hdr.src_addr.0);
        assert_eq!(link::FrameType::IP, frame_hdr.ty);
    }
}
