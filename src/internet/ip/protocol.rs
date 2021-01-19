use std::io::Cursor;

use byteorder::{BigEndian, ReadBytesExt};

use crate::internet;

const IP_BROADCAST_ADDRESS: internet::IPv4Addr = internet::IPv4Addr(0xffffffff);

/// Internet Protocol
pub struct IP();

/// プロトコルの動作モード
enum ProcessMode {
    /// 自身に向けられたパケットを受理した場合
    Me,
    /// 他のホストに向けられたパケットの場合
    AnotherHost,
}

#[derive(thiserror::Error, Debug)]
pub enum IPError {
    #[error("frame type is ipv4 but version isn't 4 in vhl")]
    NotIPv4Packet,
    #[error("invalid packet length was found")]
    InvalidPacketLength,
    #[error("invalid checksum found")]
    InvalidChecksum,
    #[error("packet was dead (TTL=0)")]
    PacketWasDead,
}

impl internet::InternetLayer for IP {
    type PacketHeader = internet::IPHeader;

    fn run(&self, buf: &[u8]) -> Result<(internet::IPHeader, Vec<u8>), Box<dyn std::error::Error>> {
        let static_ip_addr = internet::IPv4Addr::from([192, 168, 7, 167]);
        let network_mask = internet::IPv4Addr::from([255, 255, 255, 0]);
        let packet_hdr = self.parse_packet(&buf)?;
        let header_length = packet_hdr.ihl_bytes_from_vhl();
        let rest = buf[header_length..].to_vec();

        let mode =
            self.validate_ip_packet(buf, &packet_hdr, &static_ip_addr, &network_mask, buf.len())?;

        match mode {
            ProcessMode::AnotherHost => {
                // TODO: ルータとしての使用を想定していないので，今の所無視
            }
            ProcessMode::Me => {
                if packet_hdr.is_fragmented() {
                    // TODO: フラグメントパケットの処理
                    todo!()
                }

                // TODO: 上位プロトコルへデータの転送
            }
        }

        Ok((packet_hdr, rest))
    }
}

impl IP {
    /// IPv4 Packetのパース
    fn parse_packet(&self, buf: &[u8]) -> Result<internet::IPHeader, Box<dyn std::error::Error>> {
        let mut reader = Cursor::new(buf);
        let mut packet_hdr: internet::IPHeader = Default::default();

        packet_hdr.version_ihl = reader.read_u8()?;
        packet_hdr.type_of_service = reader.read_u8()?;
        packet_hdr.total_length = reader.read_u16::<BigEndian>()?;
        packet_hdr.identification = reader.read_u16::<BigEndian>()?;
        packet_hdr.flg_offset = reader.read_u16::<BigEndian>()?;
        packet_hdr.time_to_live = reader.read_u8()?;

        packet_hdr.protocol = internet::TransportType::from(reader.read_u8()?);
        packet_hdr.checksum = reader.read_u16::<BigEndian>()?;
        packet_hdr.src_addr = internet::IPv4Addr(reader.read_u32::<BigEndian>()?);
        packet_hdr.dst_addr = internet::IPv4Addr(reader.read_u32::<BigEndian>()?);

        Ok(packet_hdr)
    }

    fn validate_ip_packet(
        &self,
        raw_packet: &[u8],
        packet_hdr: &internet::IPHeader,
        ip_addr: &internet::IPv4Addr,
        network_mask: &internet::IPv4Addr,
        ip_packet_len: usize,
    ) -> Result<ProcessMode, Box<dyn std::error::Error>> {
        if packet_hdr.version_from_vhl() != 4 {
            return Err(Box::new(IPError::NotIPv4Packet));
        }

        // ヘッダに格納されている"IPパケットヘッダ長" もしくは "パケットの全長"が
        // 実際のバッファサイズより大きければエラーとする
        if ip_packet_len < (packet_hdr.ihl_bytes_from_vhl())
            || ip_packet_len < packet_hdr.total_length as usize
        {
            return Err(Box::new(IPError::InvalidPacketLength));
        }

        if self.calculate_checksum(&packet_hdr, raw_packet)? != 0 {
            return Err(Box::new(IPError::InvalidChecksum));
        }

        if packet_hdr.time_to_live == 0 {
            return Err(Box::new(IPError::PacketWasDead));
        }

        // ホストに向けられたパケットであればOK
        if ip_addr == &packet_hdr.dst_addr {
            return Ok(ProcessMode::Me);
        }

        // ブロードキャストパケットであるかのチェック
        if ip_addr == &ip_addr.to_broadcast(network_mask) || ip_addr == &IP_BROADCAST_ADDRESS {
            return Ok(ProcessMode::Me);
        }

        Ok(ProcessMode::AnotherHost)
    }

    /// チェックサムの計算
    /// See also [Header checksum](https://tools.ietf.org/html/rfc791#section-3.1)
    fn calculate_checksum(
        &self,
        packet_hdr: &internet::IPHeader,
        buf: &[u8],
    ) -> Result<u16, Box<dyn std::error::Error>> {
        let mut sum: u32 = 0;
        let mut size = packet_hdr.ihl_bytes_from_vhl() as u16;
        let mut reader = Cursor::new(buf);

        loop {
            if size <= 1 {
                break;
            }

            sum += reader.read_u16::<BigEndian>()? as u32;
            if sum & 0x80000000 != 0 {
                sum = (sum & 0xffff) + (sum >> 16);
            }
            size -= 2;
        }

        if size == 1 {
            sum += reader.read_u8()? as u32;
        }

        loop {
            if (sum >> 16) == 0 {
                break;
            }
            sum = (sum & 0xffff) + (sum >> 16);
        }

        Ok(!(sum as u16))
    }
}

#[cfg(test)]
mod protocol_tests {
    // use super::*;

    #[test]
    fn parse_packet_test() {
        /*
        0x0000:  45c0 00b0 ef80 0000 4001 f977 c0a8 07a3
        0x0010:  c0a8 07a1 0303 8e23 0000 0000 4500 0094
        0x0020:  5195 0000 8011 582f c0a8 07a1 c0a8 07a3
        0x0030:  0035
        */
    }
}
