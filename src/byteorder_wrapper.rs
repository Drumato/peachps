use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};

pub fn read_u8<E>(reader: &mut Cursor<&[u8]>, err: E) -> Result<u8, E>
where
    E: std::error::Error + Copy,
{
    reader.read_u8().map_err(|_e| err)
}

pub fn read_u16_as_be<E>(reader: &mut Cursor<&[u8]>, err: E) -> Result<u16, E>
where
    E: std::error::Error + Copy,
{
    reader.read_u16::<BigEndian>().map_err(|_e| err)
}

pub fn read_u32_as_be<E>(reader: &mut Cursor<&[u8]>, err: E) -> Result<u32, E>
where
    E: std::error::Error + Copy,
{
    reader.read_u32::<BigEndian>().map_err(|_e| err)
}
pub fn write_u8<E>(buf: &mut Vec<u8>, v: u8, err: E) -> Result<(), E>
where
    E: std::error::Error + Copy,
{
    buf.write_u8(v).map_err(|_e| err)
}

pub fn write_u16_as_be<E>(buf: &mut Vec<u8>, bytes: u16, err: E) -> Result<(), E>
where
    E: std::error::Error + Copy,
{
    buf.write_u16::<BigEndian>(bytes).map_err(|_e| err)
}
pub fn write_u32_as_be<E>(buf: &mut Vec<u8>, bytes: u32, err: E) -> Result<(), E>
where
    E: std::error::Error + Copy,
{
    buf.write_u32::<BigEndian>(bytes).map_err(|_e| err)
}

pub fn write_u48_as_be<E>(buf: &mut Vec<u8>, bytes: u64, err: E) -> Result<(), E>
where
    E: std::error::Error + Copy,
{
    buf.write_u48::<BigEndian>(bytes).map_err(|_e| err)
}
