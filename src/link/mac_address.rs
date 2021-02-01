use std::io::Cursor;

use crate::byteorder_wrapper;

pub type RawMacAddress = [u8; 6];
pub const BLOADCAST_MAC_ADDRESS: MacAddress = MacAddress([0xff; 6]);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone)]
pub struct MacAddress(pub RawMacAddress);

impl MacAddress {
    pub fn from_cursor<E>(reader: &mut Cursor<&[u8]>, err: E) -> Result<Self, E>
    where
        E: std::error::Error + Copy,
    {
        let mut addr = [0x00; 6];
        for i in 0..6 {
            addr[i] = byteorder_wrapper::read_u8(reader, err)?;
        }

        Ok(Self(addr))
    }

    pub fn to_bytes<E>(&self, err: E) -> Result<Vec<u8>, E>
    where
        E: std::error::Error + Copy,
    {
        let mut buf = Vec::new();
        byteorder_wrapper::write_u48_as_be(&mut buf, (*self).into(), err)?;
        Ok(buf)
    }
}

impl std::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let addr_str = self
            .0
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<String>>()
            .join(":");

        write!(f, "{}", addr_str)
    }
}

impl From<[u8; 6]> for MacAddress {
    fn from(addr: [u8; 6]) -> Self {
        Self(addr)
    }
}

impl Into<u64> for MacAddress {
    fn into(self) -> u64 {
        let mut v = 0;
        for (i, b) in self.0.iter().enumerate() {
            v |= (b >> (i * 8)) as u64;
        }
        v
    }
}
impl Default for MacAddress {
    fn default() -> Self {
        Self([0x00; 6])
    }
}

#[cfg(test)]
mod display_tests {
    use super::*;

    #[test]
    fn display_address_test() {
        let addr = MacAddress([12, 34, 56, 78, 90, 12]);

        assert_eq!("0c:22:38:4e:5a:0c", addr.to_string());
    }
}
