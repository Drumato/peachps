use std::sync::{Arc, Mutex};

use super::FrameHeader;
use crate::{internet::InternetProtocol, link::MacAddress};
use crate::{link::LinkProtocolError, network_device, option};
pub fn rx(buf: &[u8]) -> Result<(FrameHeader, Vec<u8>), LinkProtocolError> {
    let frame_hdr = FrameHeader::new_from_bytes(buf, LinkProtocolError::CannotParseFrameHeader)?;
    let (_, rest) = buf.split_at(FrameHeader::LENGTH);
    Ok((frame_hdr, rest.to_vec()))
}

pub fn tx<ND: network_device::NetworkDevice>(
    opt: Arc<option::PeachPSOption>,
    dev: Arc<Mutex<ND>>,
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

    dev.lock().unwrap().write(&ethernet_frame)?;

    Ok(())
}
