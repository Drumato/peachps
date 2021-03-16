use crate::interface;
use crate::protocol::ether::Frame;

use super::{EthernetError, MacAddress};

pub fn input(dev: interface::tap::TAPDevice) -> Result<(Frame, Vec<u8>), EthernetError> {
    let mut buffer = [0; 1024];

    let nbytes = match dev.read(&mut buffer) {
        Ok(nbytes) => nbytes,
        Err(_e) => {
            return Err(EthernetError::Read);
        }
    };

    if nbytes == 0 {
        return Err(EthernetError::EOF);
    }

    let ether_frame = match Frame::from_bytes(&buffer) {
        Ok(frame) => frame,
        Err(_e) => {
            return Err(EthernetError::ParseBytesAsFrame);
        }
    };

    eprintln!("{}", ether_frame);

    if ether_frame.dst_addr != dev.addr || ether_frame.dst_addr != MacAddress::BLOADCAST {
        return Err(EthernetError::Ignore);
    }

    let (_, rest) = buffer.split_at(Frame::LENGTH);

    Ok((ether_frame, rest.to_vec()))
}
