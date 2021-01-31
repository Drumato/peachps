use super::FrameHeader;
use crate::{byteorder_wrapper, link::LinkProtocolError, network_device, option};
use crate::{internet::InternetProtocol, link::MacAddress};
use std::io::Cursor;

pub fn rx(buf: &[u8]) -> Result<(FrameHeader, Vec<u8>), LinkProtocolError> {
    let frame_hdr = parse_ethernet_frame(buf)?;
    let (_, rest) = buf.split_at(FrameHeader::LENGTH);
    Ok((frame_hdr, rest.to_vec()))
}

pub fn tx<ND: network_device::NetworkDevice>(
    opt: option::PeachPSOption,
    dev: &mut ND,
    ip_type: InternetProtocol,
    dst_addr: MacAddress,
    mut payload: Vec<u8>,
) -> Result<(), LinkProtocolError> {
    let mut ethernet_frame = Vec::<u8>::new();
    let frame_hdr = FrameHeader {
        dst_addr,
        src_addr: opt.dev_addr,
        ty: ip_type,
    };
    ethernet_frame.append(&mut frame_hdr.to_bytes(LinkProtocolError::CannotConstructFrame)?);
    ethernet_frame.append(&mut payload);

    dev.write(&ethernet_frame)?;

    Ok(())
}

fn parse_ethernet_frame(buf: &[u8]) -> Result<FrameHeader, LinkProtocolError> {
    let mut reader = Cursor::new(buf);
    let mut frame_hdr: FrameHeader = Default::default();

    for i in 0..6 {
        frame_hdr.dst_addr.0[i] =
            byteorder_wrapper::read_u8(&mut reader, LinkProtocolError::CannotParseFrameHeader)?;
    }
    for i in 0..6 {
        frame_hdr.src_addr.0[i] =
            byteorder_wrapper::read_u8(&mut reader, LinkProtocolError::CannotParseFrameHeader)?;
    }

    let frame_type =
        byteorder_wrapper::read_u16_as_be(&mut reader, LinkProtocolError::CannotParseFrameHeader)?;

    frame_hdr.ty = InternetProtocol::from(frame_type);

    Ok(frame_hdr)
}

#[cfg(test)]
mod tests {
    use crate::internet;

    use super::*;

    #[test]
    fn parse_ethernet_frame_test() {
        let raw_frame = [
            0x00, 0x15, 0x5d, 0x22, 0x1e, 0xff, 0x00, 0x15, 0x5d, 0x74, 0x4d, 0x66, 0x08, 0x00,
        ];
        let result = parse_ethernet_frame(&raw_frame);
        assert!(result.is_ok());
        let frame_hdr = result.unwrap();

        assert_eq!([0x00, 0x15, 0x5d, 0x22, 0x1e, 0xff], frame_hdr.dst_addr.0);
        assert_eq!([0x00, 0x15, 0x5d, 0x74, 0x4d, 0x66], frame_hdr.src_addr.0);
        assert_eq!(internet::InternetProtocol::IP, frame_hdr.ty);
    }
}
