use std::io::Cursor;

use crate::{
    byteorder_wrapper,
    internet::{ip::IPv4Addr, InternetProtocol, InternetProtocolError},
    link::LinkProtocol,
    option, RxResult,
};

use super::{ARPHeader, Operation};

pub fn rx(
    opt: option::PeachPSOption,
    rx_result: RxResult,
    buf: &[u8],
) -> Result<(RxResult, Vec<u8>), InternetProtocolError> {
    let arp_packet_hdr = parse_arp_packet(buf)?;
    if opt.debug {
        eprintln!("++++++++ ARP Packet ++++++++");
        eprintln!("{}", arp_packet_hdr);
    }

    todo!()
}

fn parse_arp_packet(buf: &[u8]) -> Result<ARPHeader, InternetProtocolError> {
    let mut reader = Cursor::new(buf);
    let mut packet_hdr: ARPHeader = Default::default();

    packet_hdr.link_type = match byteorder_wrapper::read_u16_as_be(
        &mut reader,
        InternetProtocolError::CannotParsePacketHeader,
    )? {
        1 => LinkProtocol::Ethernet,
        n => panic!("unsupported hardware address space => {}", n),
    };
    packet_hdr.internet_type = match byteorder_wrapper::read_u16_as_be(
        &mut reader,
        InternetProtocolError::CannotParsePacketHeader,
    )? {
        0x0800 => InternetProtocol::IP,
        n => panic!("unsupported protocol address space => {}", n),
    };
    packet_hdr.link_addr_length =
        byteorder_wrapper::read_u8(&mut reader, InternetProtocolError::CannotParsePacketHeader)?;
    packet_hdr.internet_addr_length =
        byteorder_wrapper::read_u8(&mut reader, InternetProtocolError::CannotParsePacketHeader)?;
    packet_hdr.operation = Operation::from(byteorder_wrapper::read_u16_as_be(
        &mut reader,
        InternetProtocolError::CannotParsePacketHeader,
    )?);

    for i in 0..6 {
        packet_hdr.src_link_addr.0[i] = byteorder_wrapper::read_u8(
            &mut reader,
            InternetProtocolError::CannotParsePacketHeader,
        )?;
    }

    packet_hdr.src_internet_addr = IPv4Addr(byteorder_wrapper::read_u32_as_be(
        &mut reader,
        InternetProtocolError::CannotParsePacketHeader,
    )?);

    for i in 0..6 {
        packet_hdr.dst_link_addr.0[i] = byteorder_wrapper::read_u8(
            &mut reader,
            InternetProtocolError::CannotParsePacketHeader,
        )?;
    }
    packet_hdr.dst_internet_addr = IPv4Addr(byteorder_wrapper::read_u32_as_be(
        &mut reader,
        InternetProtocolError::CannotParsePacketHeader,
    )?);
    Ok(packet_hdr)
}
