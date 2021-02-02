use std::io::Cursor;

use crate::byteorder_wrapper;

/// チェックサムの計算
/// See also [Header checksum](https://tools.ietf.org/html/rfc791#section-3.1)
pub fn calculate_checksum_u16<E>(buf: &[u8], length: u16, err: E) -> Result<u16, E>
where
    E: std::error::Error + Copy,
{
    let mut sum: u32 = 0;
    let mut size = length;
    let mut reader = Cursor::new(buf);

    loop {
        if size <= 1 {
            break;
        }

        sum += byteorder_wrapper::read_u16_as_be(&mut reader, err)? as u32;

        if sum & 0x80000000 != 0 {
            sum = (sum & 0xffff) + sum.checked_shr(16).unwrap();
        }
        size -= 2;
    }

    if size == 1 {
        sum += byteorder_wrapper::read_u8(&mut reader, err)? as u32;
    }

    loop {
        if sum.checked_shr(16).unwrap() == 0 {
            break;
        }
        sum = (sum & 0xffff) + sum.checked_shr(16).unwrap();
    }

    Ok(!(sum as u16))
}
