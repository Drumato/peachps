use super::FrameHeader;
use crate::{internet::InternetProtocol, link::MacAddress, Items};
use crate::{link::LinkProtocolError, network_device};
pub async fn rx(buf: &[u8]) -> Result<(FrameHeader, Vec<u8>), LinkProtocolError> {
    let frame_hdr = FrameHeader::new_from_bytes(buf, LinkProtocolError::CannotParseFrameHeader)?;
    let (_, rest) = buf.split_at(FrameHeader::LENGTH);
    Ok((frame_hdr, rest.to_vec()))
}

pub async fn tx<'a, ND: network_device::NetworkDevice>(
    table: &'a Items<ND>,
    ip_type: InternetProtocol,
    dst_addr: MacAddress,
    mut payload: Vec<u8>,
) -> Result<(), LinkProtocolError> {
    let mut ethernet_frame = Vec::<u8>::new();
    let frame_hdr = FrameHeader {
        dst_addr,
        src_addr: table.opt.dev_addr,
        ty: ip_type,
    };
    ethernet_frame.append(&mut frame_hdr.to_bytes(LinkProtocolError::CannotConstructFrame)?);
    ethernet_frame.append(&mut payload);

    if let Ok(ref mut dev) = table.dev.lock() {
        dev.write(&ethernet_frame).await?;
    }

    Ok(())
}
