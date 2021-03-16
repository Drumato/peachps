use byteorder::{BigEndian, ReadBytesExt};
use std::{fmt::Display, io::Cursor};

use super::Type;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Frame {
    pub dst_addr: MacAddress,
    pub src_addr: MacAddress,
    pub protocol: Type,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MacAddress([u8; 6]);

impl Into<u64> for MacAddress {
    fn into(self) -> u64 {
        let addr = [
            0x00, 0x00, self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5],
        ];
        u64::from_be_bytes(addr)
    }
}

impl From<[i8; 14]> for MacAddress {
    fn from(v: [i8; 14]) -> Self {
        MacAddress([
            v[0] as u8, v[1] as u8, v[2] as u8, v[3] as u8, v[4] as u8, v[5] as u8,
        ])
    }
}

impl Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

impl Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Dst addr: {}", self.dst_addr)?;
        writeln!(f, "Src addr: {}", self.src_addr)?;
        writeln!(f, "Protocol: {}", self.protocol)
    }
}

impl Frame {
    /// Ethernet Frameの長さ
    pub const LENGTH: usize = 14;

    pub fn from_bytes(buf: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut cursor = Cursor::new(buf);

        let dst_addr = MacAddress::from_cursor(&mut cursor)?;
        let src_addr = MacAddress::from_cursor(&mut cursor)?;
        let protocol = cursor.read_u16::<BigEndian>()?;

        Ok(Self {
            dst_addr,
            src_addr,
            protocol: Type::from(protocol),
        })
    }
}

impl MacAddress {
    pub const BLOADCAST: MacAddress = MacAddress([0xff; 6]);
    pub fn from_cursor(reader: &mut Cursor<&[u8]>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut addr = [0x00; 6];
        for i in 0..6 {
            addr[i] = reader.read_u8()?;
        }

        Ok(Self(addr))
    }
}
