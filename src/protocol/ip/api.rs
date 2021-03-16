use super::{IPError, IPv4Addr, PacketHeader};
use crate::interface::tap;
use crate::protocol::ether;

pub fn input(dev: tap::TAPDevice, host_ip: IPv4Addr) -> Result<(PacketHeader, Vec<u8>), IPError> {
    let (frame, rest) = ether::input(dev).map_err(|e| match e {
        ether::EthernetError::Ignore => IPError::Ignore,
        _ => IPError::EthernetError { err: e },
    })?;

    if frame.protocol != ether::Type::IP {
        return Err(IPError::Ignore);
    }

    let packet = PacketHeader::from_bytes(&rest).map_err(|_e| IPError::ParseBytesAsPacket)?;

    let (_, rest) = rest.split_at(packet.ihl_as_octets());

    Ok((packet, rest.to_vec()))
}
