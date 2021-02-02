use std::io::Cursor;

use crate::{byteorder_wrapper, transport::TransportHeader};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MessageHeader {
    pub ty: MessageType,
    pub code: u8,
    pub checksum: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessageType {
    /// エコー応答
    EchoReply,
    /// 目的地到達不能
    DestinationUnreachable,
    /// 経路変更
    Redirect,
    /// エコー要求
    EchoRequest,
    /// 時間超過
    TimeExceeded,
}

impl MessageHeader {
    pub const LENGTH: usize = 4;

    pub fn new_from_bytes<E>(buf: &[u8], err: E) -> Result<Self, E>
    where
        E: std::error::Error + Copy,
    {
        let mut reader = Cursor::new(buf);
        let mut message_header: MessageHeader = Default::default();

        let message_type = byteorder_wrapper::read_u8(&mut reader, err)?;

        message_header.ty = MessageType::from(message_type);
        message_header.code = byteorder_wrapper::read_u8(&mut reader, err)?;
        message_header.checksum = byteorder_wrapper::read_u16_as_be(&mut reader, err)?;

        Ok(message_header)
    }

    pub fn to_bytes<E>(&self, err: E) -> Result<Vec<u8>, E>
    where
        E: std::error::Error + Copy,
    {
        let mut buf = Vec::<u8>::new();
        byteorder_wrapper::write_u8(&mut buf, self.ty.into(), err)?;
        byteorder_wrapper::write_u8(&mut buf, self.code, err)?;
        byteorder_wrapper::write_u16_as_be(&mut buf, self.checksum, err)?;
        Ok(buf)
    }
}

impl TransportHeader for MessageHeader {}

impl Default for MessageHeader {
    fn default() -> Self {
        Self {
            ty: MessageType::EchoReply,
            code: 0,
            checksum: 0,
        }
    }
}

impl std::fmt::Display for MessageHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Type: {}", self.ty)?;
        writeln!(f, "Code: {}", self.code)?;
        writeln!(f, "Checksum: {}", self.checksum)
    }
}

impl std::fmt::Display for MessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let type_str = match self {
            MessageType::EchoReply => "Echo Reply",
            MessageType::DestinationUnreachable => "Destination Unreachable",
            MessageType::Redirect => "Redirect",
            MessageType::EchoRequest => "Echo Request",
            MessageType::TimeExceeded => "Time Exceeded",
        };
        write!(f, "{}", type_str)
    }
}

impl From<u8> for MessageType {
    fn from(v: u8) -> Self {
        match v {
            0 => MessageType::EchoReply,
            3 => MessageType::DestinationUnreachable,
            5 => MessageType::Redirect,
            8 => MessageType::EchoRequest,
            11 => MessageType::TimeExceeded,
            _ => unimplemented!(),
        }
    }
}

impl Into<u8> for MessageType {
    fn into(self) -> u8 {
        match self {
            MessageType::EchoReply => 0,
            MessageType::DestinationUnreachable => 3,
            MessageType::Redirect => 5,
            MessageType::EchoRequest => 8,
            MessageType::TimeExceeded => 11,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::transport::TransportProtocolError;

    use super::*;

    #[test]
    fn parse_icmp_message_test() {
        let raw_message = [0x00, 0x00, 0x55, 0x49];
        let result = MessageHeader::new_from_bytes(
            &raw_message,
            TransportProtocolError::CannotParseICMPMessage,
        );
        assert!(result.is_ok());
        let message_hdr = result.unwrap();
        assert_eq!(MessageType::EchoReply, message_hdr.ty);
        assert_eq!(0, message_hdr.code);
        assert_eq!(0x5549, message_hdr.checksum);
    }
}
