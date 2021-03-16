use std::{fmt::Display, io::Cursor};

use byteorder::{BigEndian, ReadBytesExt};

use crate::calculator;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct PacketHeader {
    version_and_ihl: u8,
    pub type_of_service: u8,
    pub total_length: u16,
    pub identification: u32,
    flags_and_offset: u16,
    pub time_to_live: u16,
    pub protocol: TransportType,
    pub checksum: u16,
    pub src_addr: IPv4Addr,
    pub dst_addr: IPv4Addr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TransportType {
    TCP,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IPv4Addr(pub [u8; 4]);

impl Display for PacketHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Version         : {}", self.version())?;
        writeln!(f, "Header Length   : {}", self.ihl_as_octets())?;
        writeln!(f, "Type of Service : {}", self.type_of_service)?;
        writeln!(f, "Total Length    : {}", self.total_length)?;
        writeln!(f, "Identification  : {}", self.identification)?;
        writeln!(f, "Flags           : 0b{:03b}", self.flags())?;
        writeln!(f, "Offset          : {}", self.offset())?;
        writeln!(f, "TTL             : {}", self.time_to_live)?;
        writeln!(f, "Protocol        : {}", self.protocol)?;
        writeln!(f, "Checksum        : {}", self.checksum)?;
        writeln!(f, "Src addr        : {}", self.src_addr)?;
        writeln!(f, "Dst addr        : {}", self.dst_addr)
    }
}

impl From<u8> for TransportType {
    fn from(v: u8) -> Self {
        match v {
            6 => TransportType::TCP,
            _ => unimplemented!(),
        }
    }
}

impl Display for TransportType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                TransportType::TCP => "TCP",
            }
        )
    }
}

impl From<u32> for IPv4Addr {
    fn from(v: u32) -> Self {
        IPv4Addr([
            ((v & 0xff000000) >> 24) as u8,
            ((v & 0x00ff0000) >> 16) as u8,
            ((v & 0x0000ff00) >> 8) as u8,
            (v & 0x000000ff) as u8,
        ])
    }
}

impl Display for IPv4Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}.{}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

impl PacketHeader {
    const VERSION_MASK: u8 = 0xf0;
    const FLAGS_MASK: u16 = 0xe0;
    const LEAST_LENGTH_OCTETS: usize = 20;

    pub fn flags(&self) -> u16 {
        (self.flags_and_offset & Self::FLAGS_MASK) >> 13
    }
    pub fn offset(&self) -> u16 {
        self.flags_and_offset & !Self::FLAGS_MASK
    }

    pub fn version(&self) -> u8 {
        self.version_and_ihl & Self::VERSION_MASK
    }

    /// Internet Header Lengthの取り出し
    /// そのままの値だと32bit wordの数なので，オクテット単位に変換
    pub fn ihl_as_octets(&self) -> usize {
        calculator::double_words_as_octets((self.version_and_ihl & !Self::VERSION_MASK) as usize)
    }

    pub fn from_bytes(buf: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut cursor = Cursor::new(buf);
        let version_and_ihl = cursor.read_u8()?;
        let type_of_service = cursor.read_u8()?;
        let total_length = cursor.read_u16::<BigEndian>()?;
        let identification: u32 = cursor.read_u32::<BigEndian>()?;
        let flags_and_offset = cursor.read_u16::<BigEndian>()?;
        let time_to_live = cursor.read_u16::<BigEndian>()?;
        let protocol = TransportType::from(cursor.read_u8()?);
        let checksum = cursor.read_u16::<BigEndian>()?;
        let src_addr = IPv4Addr::from(cursor.read_u32::<BigEndian>()?);
        let dst_addr = IPv4Addr::from(cursor.read_u32::<BigEndian>()?);

        let base = Self {
            version_and_ihl,
            type_of_service,
            total_length,
            identification,
            flags_and_offset,
            time_to_live,
            protocol,
            checksum,
            src_addr,
            dst_addr,
        };

        // OPTIONは全部読み飛ばす
        for _ in 0..(base.ihl_as_octets() - PacketHeader::LEAST_LENGTH_OCTETS) {
            let _ = cursor.read_u8()?;
        }
        Ok(base)
    }
}
