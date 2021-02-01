use super::{MessageHeader, MessageType};
use crate::{
    byteorder_wrapper,
    internet::{self},
    network_device, option,
    transport::{TransportProtocol, TransportProtocolError},
    RxResult,
};

pub fn rx<ND: network_device::NetworkDevice>(
    opt: option::PeachPSOption,
    dev: &mut ND,
    rx_result: RxResult,
    buf: &[u8],
) -> Result<(MessageHeader, Vec<u8>), TransportProtocolError> {
    let header =
        MessageHeader::new_from_bytes(buf, TransportProtocolError::CannotParseICMPMessage)?;

    if opt.debug {
        eprintln!("++++++++ ICMP Message ++++++++");
        eprintln!("{}", header);
    }

    let (_, rest) = buf.split_at(MessageHeader::LENGTH);

    if header.ty == MessageType::EchoRequest {
        // srcとdstの関係が逆になるので注意
        tx(opt, dev, MessageType::EchoReply, header.code, rx_result)?;
    }

    Ok((header, rest.to_vec()))
}

pub fn tx<ND: network_device::NetworkDevice>(
    opt: option::PeachPSOption,
    dev: &mut ND,
    msg_type: MessageType,
    code: u8,
    rx_result: RxResult,
) -> Result<(), TransportProtocolError> {
    let mut icmp_message = Vec::<u8>::new();
    byteorder_wrapper::write_u8(
        &mut icmp_message,
        msg_type.into(),
        TransportProtocolError::CannotConstructICMPMessage,
    )?;
    byteorder_wrapper::write_u8(
        &mut icmp_message,
        code,
        TransportProtocolError::CannotConstructICMPMessage,
    )?;
    byteorder_wrapper::write_u16_as_be(
        &mut icmp_message,
        0x00,
        TransportProtocolError::CannotConstructICMPMessage,
    )?;

    match internet::ip::tx(opt, dev, TransportProtocol::ICMP, rx_result, icmp_message) {
        Ok(_) => Ok(()),
        Err(e) => Err(TransportProtocolError::IPError { e }),
    }
}
