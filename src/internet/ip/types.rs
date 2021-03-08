use std::io::Cursor;

use crate::{byteorder_wrapper, transport};

/// vhl領域のうちversionが該当する部分のマスク
const VHL_VERSION_MASK: u8 = 0xf0;
/// vhl領域のうちihlが該当する部分のマスク
const VHL_IHL_MASK: u8 = 0x0f;
/// flag_and_offset領域のうちフラグが該当する部分のマスク
const FLGOFFSET_FLAG_MASK: u16 = 0xe000;
/// flag_and_offset領域のうちオフセットが該当する部分のマスク
const FLGOFFSET_OFFSET_MASK: u16 = 0x1fff;
/// ブロードキャストアドレス
pub const IP_BROADCAST_ADDRESS: IPv4Addr = IPv4Addr(0xffffffff);

/// IPパケットのヘッダ構造体
pub struct IPHeader {
    /// 上位4ビット: version, 下位4ビット: internet_header_length
    pub version_ihl: u8,
    /// IPパケットの優先順位等の情報．
    pub type_of_service: u8,
    /// パケットの全長．
    pub total_length: u16,
    /// フラグメントされたパケットの識別情報
    pub identification: u16,
    /// 上位3ビット: フラグ, 下位13ビット: フラグメントオフセット
    /// フラグはフラグメンテーションにおける制御情報，
    /// オフセットは元パケットのどの部分に該当するかを意味
    pub flg_offset: u16,
    /// パケットの生存情報を示す．
    /// ホップ可能数と読み替えることもできる
    pub time_to_live: u8,
    /// トランスポート層のプロトコルを示す．
    pub protocol: transport::TransportProtocol,
    /// IPヘッダのエラーチェックに使用するチェックサム．
    pub checksum: u16,
    /// 送信元IPアドレス
    pub src_addr: IPv4Addr,
    /// 宛先IPアドレス
    pub dst_addr: IPv4Addr,
}

/// IPv4 Address
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Copy, Clone, Hash)]
pub struct IPv4Addr(pub u32);

impl IPv4Addr {
    pub const ANY: Self = Self(0);

    pub fn to_bytes<E>(&self, err: E) -> Result<Vec<u8>, E>
    where
        E: std::error::Error + Copy,
    {
        let mut buf = Vec::new();
        byteorder_wrapper::write_u32_as_be(&mut buf, self.0, err)?;
        Ok(buf)
    }
}

impl IPHeader {
    /// IPヘッダが持つ最低の長さ
    pub const LEAST_LENGTH: u8 = 20;
    /// ラストフラグメント以外のパケットにつけられる
    const MORE_FRAGMENTS_FLAG: u16 = 0x2000;
    pub const VERSION4: u8 = 4;

    pub fn new_from_bytes<E>(buf: &[u8], err: E) -> Result<Self, E>
    where
        E: std::error::Error + Copy,
    {
        let mut reader = Cursor::new(buf);
        let mut packet_hdr: IPHeader = Default::default();

        packet_hdr.version_ihl = byteorder_wrapper::read_u8(&mut reader, err)?;

        packet_hdr.type_of_service = byteorder_wrapper::read_u8(&mut reader, err)?;
        packet_hdr.total_length = byteorder_wrapper::read_u16_as_be(&mut reader, err)?;
        packet_hdr.identification = byteorder_wrapper::read_u16_as_be(&mut reader, err)?;
        packet_hdr.flg_offset = byteorder_wrapper::read_u16_as_be(&mut reader, err)?;
        packet_hdr.time_to_live = byteorder_wrapper::read_u8(&mut reader, err)?;
        packet_hdr.protocol =
            transport::TransportProtocol::from(byteorder_wrapper::read_u8(&mut reader, err)?);
        packet_hdr.checksum = byteorder_wrapper::read_u16_as_be(&mut reader, err)?;
        packet_hdr.src_addr = IPv4Addr(byteorder_wrapper::read_u32_as_be(&mut reader, err)?);
        packet_hdr.dst_addr = IPv4Addr(byteorder_wrapper::read_u32_as_be(&mut reader, err)?);

        // オプションは読み飛ばす
        if packet_hdr.ihl_bytes_from_vhl() > Self::LEAST_LENGTH {
            for _ in 0..(packet_hdr.ihl_bytes_from_vhl() - Self::LEAST_LENGTH) {
                let _ = byteorder_wrapper::read_u8(&mut reader, err)?;
            }
        }

        Ok(packet_hdr)
    }

    pub fn to_bytes<E>(&self, err: E) -> Result<Vec<u8>, E>
    where
        E: std::error::Error + Copy,
    {
        let mut buf = Vec::new();
        byteorder_wrapper::write_u8(&mut buf, self.version_ihl, err)?;
        byteorder_wrapper::write_u8(&mut buf, self.type_of_service, err)?;
        byteorder_wrapper::write_u16_as_be(&mut buf, self.total_length, err)?;
        byteorder_wrapper::write_u16_as_be(&mut buf, self.identification, err)?;
        byteorder_wrapper::write_u16_as_be(&mut buf, self.flg_offset, err)?;
        byteorder_wrapper::write_u8(&mut buf, self.time_to_live, err)?;
        byteorder_wrapper::write_u8(&mut buf, self.protocol.into(), err)?;
        byteorder_wrapper::write_u16_as_be(&mut buf, self.checksum, err)?;
        buf.append(&mut self.src_addr.to_bytes(err)?);
        buf.append(&mut self.dst_addr.to_bytes(err)?);

        Ok(buf)
    }

    /// vhl領域からversionだけを取り出す
    pub fn version_from_vhl(&self) -> u8 {
        (self.version_ihl & VHL_VERSION_MASK)
            .checked_shr(4)
            .unwrap()
    }
    /// vhl領域からihlだけを取り出す
    /// IPヘッダ長を4で割った値(32bitワードの数)であるため，
    /// 2ビット左シフトしてバイト単位に直す
    pub fn ihl_bytes_from_vhl(&self) -> u8 {
        (self.version_ihl & VHL_IHL_MASK).checked_shl(2).unwrap()
    }

    /// flag_and_offset領域からフラグだけを取り出す
    pub fn flag_from_flg_offset(&self) -> u16 {
        (self.flg_offset & FLGOFFSET_FLAG_MASK)
            .checked_shr(13)
            .unwrap()
    }

    /// flag_and_offset領域からフラグメントオフセットだけを取り出す
    pub fn offset_from_flg_offset(&self) -> u16 {
        self.flg_offset & FLGOFFSET_OFFSET_MASK
    }

    /// フラグメンテーションされたパケットかどうかチェック
    /// See also [RFC](https://tools.ietf.org/html/rfc791#page-13)
    pub fn is_fragmented(&self) -> bool {
        self.is_wip_fragment() || self.is_last_fragment()
    }

    /// ラストフラグメント以外のフラグメンテーションされたパケットかどうかチェック
    fn is_wip_fragment(&self) -> bool {
        (self.flg_offset & Self::MORE_FRAGMENTS_FLAG) != 0
    }

    fn is_last_fragment(&self) -> bool {
        self.offset_from_flg_offset() != 0
    }
}

impl Default for IPHeader {
    fn default() -> Self {
        Self {
            version_ihl: 0,
            type_of_service: 0,
            total_length: 0,
            identification: 0,
            flg_offset: 0,
            time_to_live: 0,
            protocol: transport::TransportProtocol::ICMP,
            checksum: 0,
            src_addr: Default::default(),
            dst_addr: Default::default(),
        }
    }
}

impl std::fmt::Display for IPHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "version: {}", self.version_from_vhl())?;
        writeln!(f, "header_length (bytes): {}", self.ihl_bytes_from_vhl())?;
        writeln!(f, "service_type: {}", self.type_of_service)?;
        writeln!(f, "total_length (bytes): {}", self.total_length)?;
        writeln!(f, "identification: {}", self.identification)?;
        writeln!(f, "flag: {:b}", self.flag_from_flg_offset())?;
        writeln!(f, "fragment offset: {}", self.offset_from_flg_offset())?;
        writeln!(f, "time to live: {}", self.time_to_live)?;
        writeln!(f, "transport protocol: {}", self.protocol)?;
        writeln!(f, "checksum: {}", self.checksum)?;
        writeln!(f, "src_addr: {}", self.src_addr)?;
        writeln!(f, "dst_addr: {}", self.dst_addr)?;

        Ok(())
    }
}

impl IPv4Addr {
    /// 自身のIPアドレスから，ホスト部をすべて1にしたものを返す
    pub fn to_broadcast(&self, network_mask: Self) -> Self {
        let host_mask = !network_mask.0;
        Self(self.0 | host_mask)
    }
}
impl From<&str> for IPv4Addr {
    fn from(s: &str) -> Self {
        let mut iter = s.split(".").map(|v| u32::from_str_radix(v, 10).unwrap());
        Self(
            iter.next().unwrap() << 24
                | iter.next().unwrap() << 16
                | iter.next().unwrap() << 8
                | iter.next().unwrap(),
        )
    }
}

impl std::fmt::Display for IPv4Addr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let addr_str = self
            .0
            .to_be_bytes()
            .iter()
            .map(|b| format!("{}", b))
            .collect::<Vec<String>>()
            .join(".");
        write!(f, "{}", addr_str)
    }
}

impl Default for IPv4Addr {
    fn default() -> Self {
        Self(0x00)
    }
}

impl From<[u8; 4]> for IPv4Addr {
    fn from(v: [u8; 4]) -> Self {
        let v: Vec<u32> = v.iter().map(|b| *b as u32).collect();
        IPv4Addr(v[0] << 24 | v[1] << 16 | v[2] << 8 | v[3])
    }
}

#[cfg(test)]
mod tests {
    use transport::TransportProtocol;

    use crate::internet::InternetProtocolError;

    use super::*;

    #[test]
    fn display_address_test() {
        let addr = IPv4Addr::from([192, 168, 11, 24]);
        assert_eq!("192.168.11.24", addr.to_string());
        let addr = IPv4Addr(0xc0a80b03);
        assert_eq!("192.168.11.3", addr.to_string());
    }

    #[test]
    fn address_from_str_test() {
        let addr = IPv4Addr::from("192.168.11.24");
        assert_eq!(IPv4Addr::from([192, 168, 11, 24]), addr);
    }

    #[test]
    fn parse_ip_packet_test() {
        let raw_packet = [
            0x45, 0x00, 0x00, 0x3c, 0x19, 0x17, 0x00, 0x00, 0x40, 0x01, 0xca, 0x55, 0xc0, 0xa8,
            0x0b, 0x01, 0xc0, 0xa8, 0x0b, 0x03,
        ];
        let result =
            IPHeader::new_from_bytes(&raw_packet, InternetProtocolError::CannotParsePacketHeader);
        assert!(result.is_ok());
        let packet_hdr = result.unwrap();
        assert_eq!(4, packet_hdr.version_from_vhl());
        assert_eq!(20, packet_hdr.ihl_bytes_from_vhl());
        assert_eq!(0, packet_hdr.type_of_service);
        assert_eq!(60, packet_hdr.total_length);
        assert_eq!(0x1917, packet_hdr.identification);
        assert_eq!(0x0, packet_hdr.flag_from_flg_offset());
        assert_eq!(0x0, packet_hdr.offset_from_flg_offset());
        assert_eq!(64, packet_hdr.time_to_live);
        assert_eq!(TransportProtocol::ICMP, packet_hdr.protocol);
        assert_eq!(0xca55, packet_hdr.checksum);
        assert_eq!("192.168.11.1", packet_hdr.src_addr.to_string());
        assert_eq!("192.168.11.3", packet_hdr.dst_addr.to_string());
    }
}
