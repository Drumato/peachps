use std::collections::HashMap;

use super::{Message, MessageType};
use crate::{
    checksum,
    internet::{self},
    link, network_device, option,
    transport::{TransportProtocol, TransportProtocolError},
    RxResult,
};

pub fn rx<ND: network_device::NetworkDevice>(
    opt: option::PeachPSOption,
    dev: &mut ND,
    rx_result: RxResult,
    buf: &[u8],
    arp_cache: &mut HashMap<internet::ip::IPv4Addr, link::MacAddress>,
) -> Result<(Message, Vec<u8>), TransportProtocolError> {
    let header = Message::new_from_bytes(buf, TransportProtocolError::CannotParseICMPMessage)?;

    if opt.debug {
        eprintln!("++++++++ ICMP Message ++++++++");
        eprintln!("{}", header);
    }

    let (_, rest) = buf.split_at(Message::LENGTH);

    if header.ty == MessageType::EchoRequest {
        // srcとdstの関係が逆になるので注意
        tx(
            opt,
            dev,
            MessageType::EchoReply,
            header,
            rx_result,
            arp_cache,
        )?;
    }

    Ok((header, rest.to_vec()))
}

pub fn tx<ND: network_device::NetworkDevice>(
    opt: option::PeachPSOption,
    dev: &mut ND,
    msg_type: MessageType,
    received_msg: Message,
    rx_result: RxResult,
    arp_cache: &mut HashMap<internet::ip::IPv4Addr, link::MacAddress>,
) -> Result<(), TransportProtocolError> {
    let mut icmp_message: Message = Default::default();
    icmp_message.ty = msg_type;
    icmp_message.code = received_msg.code;
    icmp_message.data = received_msg.data;

    let before_buf = icmp_message.to_bytes(TransportProtocolError::CannotConstructICMPMessage)?;
    let cksum = checksum::calculate_checksum_u16(
        &before_buf,
        before_buf.len() as u16,
        TransportProtocolError::InvalidChecksum,
    )?;
    icmp_message.checksum = cksum;

    if opt.debug {
        eprintln!("++++++++ tx icmp message ++++++++");
        eprintln!("{}", icmp_message);
    }

    match internet::ip::tx(
        opt,
        dev,
        TransportProtocol::ICMP,
        rx_result,
        icmp_message.to_bytes(TransportProtocolError::CannotConstructICMPMessage)?,
        arp_cache,
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(TransportProtocolError::IPError { e }),
    }
}
