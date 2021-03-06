use super::{Message, MessageType};
use crate::{
    checksum::calculate_checksum_u16,
    internet::{self},
    network_device,
    transport::{TransportProtocol, TransportProtocolError},
    Items, RxResult,
};

pub async fn rx<'a, ND: network_device::NetworkDevice>(
    table: &'a Items<ND>,
    rx_result: RxResult,
    buf: &[u8],
) -> Result<(Message, Vec<u8>), TransportProtocolError> {
    let mut msg = Message::new_from_bytes(
        &buf[..rx_result.message_len],
        TransportProtocolError::CannotParseICMPMessage,
    )?;
    msg.checksum = 0;
    let msg_buf = msg.to_bytes(TransportProtocolError::CannotParseICMPMessage)?;

    let _cksum = calculate_checksum_u16(
        &msg_buf,
        msg_buf.len() as u16,
        TransportProtocolError::InvalidChecksum,
    )?;

    if table.opt.debug {
        eprintln!("++++++++ rx icmp message ++++++++");
        eprintln!("{}", msg);
    }

    let (_, rest) = buf.split_at(Message::LENGTH);

    if msg.ty == MessageType::EchoRequest {
        tx(table, MessageType::EchoReply, &msg, rx_result).await?;
    }

    Ok((msg, rest.to_vec()))
}

pub async fn tx<'a, ND: network_device::NetworkDevice>(
    table: &'a Items<ND>,
    msg_type: MessageType,
    received_msg: &Message,
    rx_result: RxResult,
) -> Result<(), TransportProtocolError> {
    let mut icmp_message: Message = Default::default();
    icmp_message.ty = msg_type;
    icmp_message.code = received_msg.code;
    icmp_message.data = received_msg.data.clone();

    let before_buf = icmp_message.to_bytes(TransportProtocolError::CannotConstructICMPMessage)?;
    let cksum = calculate_checksum_u16(
        &before_buf,
        before_buf.len() as u16,
        TransportProtocolError::InvalidChecksum,
    )?;
    icmp_message.checksum = cksum;

    if table.opt.debug {
        eprintln!("++++++++ tx icmp message ++++++++");
        eprintln!("{}", icmp_message);
    }

    match internet::ip::tx(
        table,
        TransportProtocol::ICMP,
        rx_result,
        icmp_message.to_bytes(TransportProtocolError::CannotConstructICMPMessage)?,
    )
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(TransportProtocolError::IPError { e }),
    }
}
