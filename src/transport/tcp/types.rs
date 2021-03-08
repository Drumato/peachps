use std::io::Cursor;

use crate::{byteorder_wrapper, internet::ip::IPv4Addr};

pub const DATA_OFFSET_MASK: u8 = 0xf0;
pub const CONTROL_FLAG_MASK: u8 = 0xc0;

/// チェックサム計算用の疑似ヘッダ
/// セグメントにこのデータは含まれない
pub struct PseudoHeader {
    pub src_addr: IPv4Addr,
    pub dst_addr: IPv4Addr,
    pub length: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SegmentHeader {
    /// 送信元ポート
    pub src_port: u16,
    /// 宛先ポート
    pub dst_port: u16,
    /// 送信したデータの順序番号
    pub sequence: u32,
    /// 確認応答番号の値
    pub acknowledgement: u32,
    /// 後半4bitはReserved
    pub offset: u8,
    /// 前半2bitはReserved
    pub flg: u8,
    /// 受信側が一度に受信可能なデータサイズを通知するために使用
    pub window_size: u16,
    pub checksum: u16,
    /// 緊急データの開始位置
    pub urgent_pointer: u16,
}

/// TCP Control Bits.
pub mod control_flag {
    pub fn is_up(v: u8, flg: u8) -> bool {
        (v & flg) != 0
    }

    /// Urgent Pointer field significant
    pub const URG: u8 = 0x20;
    /// Acknowleagement field significant
    pub const ACK: u8 = 0x10;
    /// Push Function
    pub const PSH: u8 = 0x08;
    /// Reset the connection
    pub const RST: u8 = 0x04;
    /// Synchronize sequence numbers
    pub const SYN: u8 = 0x02;
    /// No more data from sender
    pub const FIN: u8 = 0x01;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProtocolControlBlock {
    pub state: ConnState,
    pub conn: Connection,
    /// The Initial Receive Sequence number
    pub irs: u32,
    /// The Initial Send Sequence number
    pub iss: u32,
    pub receive: ReceiveInfo,
    pub send: SendInfo,
    pub buffer: [u8; 65535],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ReceiveInfo {
    pub next: u32,
    pub window_size: u16,
    pub urgent_pointer: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SendInfo {
    pub next: u32,
    pub window_size: u16,
    pub urgent_pointer: u16,
    pub unacknowleage: u32,
    pub wl1: u32,
    pub wl2: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ConnState {
    Listen,
    Close,
    SynReceived,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EndPoint {
    pub addr: IPv4Addr,
    pub port: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Connection {
    pub local: EndPoint,
    pub foreign: EndPoint,
}

impl SegmentHeader {
    pub const LEAST_LENGTH: usize = 20;

    pub fn data_offset(&self) -> u8 {
        (self.offset & DATA_OFFSET_MASK) >> 4
    }

    pub fn control_flag(&self) -> u8 {
        self.flg & CONTROL_FLAG_MASK
    }

    pub fn flag_is_up(&self, flg: u8) -> bool {
        control_flag::is_up(self.control_flag(), flg)
    }

    pub fn new_from_bytes<E>(buf: &[u8], err: E) -> Result<Self, E>
    where
        E: std::error::Error + Copy,
    {
        let mut reader = Cursor::new(buf);
        let mut segment_header: SegmentHeader = Default::default();

        segment_header.src_port = byteorder_wrapper::read_u16_as_be(&mut reader, err)?;
        segment_header.dst_port = byteorder_wrapper::read_u16_as_be(&mut reader, err)?;
        segment_header.sequence = byteorder_wrapper::read_u32_as_be(&mut reader, err)?;
        segment_header.acknowledgement = byteorder_wrapper::read_u32_as_be(&mut reader, err)?;
        segment_header.offset = byteorder_wrapper::read_u8(&mut reader, err)?;
        segment_header.flg = byteorder_wrapper::read_u8(&mut reader, err)?;
        segment_header.window_size = byteorder_wrapper::read_u16_as_be(&mut reader, err)?;
        segment_header.checksum = byteorder_wrapper::read_u16_as_be(&mut reader, err)?;
        segment_header.urgent_pointer = byteorder_wrapper::read_u16_as_be(&mut reader, err)?;

        Ok(segment_header)
    }

    pub fn to_bytes<E>(&self, err: E) -> Result<Vec<u8>, E>
    where
        E: std::error::Error + Copy,
    {
        let mut buf = Vec::<u8>::new();
        byteorder_wrapper::write_u16_as_be(&mut buf, self.src_port, err)?;
        byteorder_wrapper::write_u16_as_be(&mut buf, self.dst_port, err)?;
        byteorder_wrapper::write_u32_as_be(&mut buf, self.sequence, err)?;
        byteorder_wrapper::write_u32_as_be(&mut buf, self.acknowledgement, err)?;
        byteorder_wrapper::write_u8(&mut buf, self.offset, err)?;
        byteorder_wrapper::write_u8(&mut buf, self.flg, err)?;
        byteorder_wrapper::write_u16_as_be(&mut buf, self.window_size, err)?;
        byteorder_wrapper::write_u16_as_be(&mut buf, self.checksum, err)?;
        byteorder_wrapper::write_u16_as_be(&mut buf, self.urgent_pointer, err)?;

        Ok(buf)
    }
}

impl std::fmt::Display for SegmentHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Src Port: {}", self.src_port)?;
        writeln!(f, "Dst Port: {}", self.dst_port)?;
        writeln!(f, "Sequence: {}", self.sequence)?;
        writeln!(f, "Acknowleage: {}", self.acknowledgement)?;
        writeln!(f, "Offset: 0x{:x}", self.data_offset())?;
        writeln!(f, "Control Flag: 0b{:b}", self.control_flag())?;
        writeln!(f, "Window Size: {}", self.window_size)?;
        writeln!(f, "Checksum: {}", self.checksum)?;
        writeln!(f, "Urgent Pointer: {}", self.urgent_pointer)
    }
}

impl Default for SegmentHeader {
    fn default() -> Self {
        Self {
            src_port: 0,
            dst_port: 0,
            sequence: 0,
            acknowledgement: 0,
            offset: 0,
            flg: 0,
            window_size: 0,
            checksum: 0,
            urgent_pointer: 0,
        }
    }
}
