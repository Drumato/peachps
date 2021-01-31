use crate::transport::TransportHeader;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MessageHeader {
    pub ty: MessageType,
    pub code: u8,
    pub checksum: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
