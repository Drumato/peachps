use crate::{internet, link};
use crate::{link::Frame, network_device as net_dev};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PeachPSError {
    #[error("end of file")]
    EOF,
}

/// プロトコルスタック
/// 静的ディスパッチにすることで実行速度を優先
pub struct ProtocolStack<ND: net_dev::NetworkDevice, P: link::LinkLayer> {
    /// ネットワークデバイスの抽象
    /// device: Box<dyn net_dev::NetworkDevice> のほうがmain.rsがキレイになるが，敢えてしていない
    pub device: ND,
    /// リンク層を扱うプロトコル
    pub link_protocol: P,
}

/// イーサネットを用いる標準的なプロトコルスタック
#[allow(dead_code)]
pub type DefaultPS<ND> = ProtocolStack<ND, link::Ethernet>;

#[allow(dead_code)]
impl<ND: net_dev::NetworkDevice, P: link::LinkLayer> ProtocolStack<ND, P> {
    pub fn new(device: ND, lp: P) -> Self {
        Self {
            device,
            link_protocol: lp,
        }
    }

    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut buf: [u8; 2048] = [0; 2048];
        let nbytes = self.device.read(&mut buf).await?;

        if nbytes == 0 {
            return Err(Box::new(PeachPSError::EOF));
        }

        let (frame_hdr, raw_ip_packet) = self.link_protocol.run(&buf)?;

        // このホストに向けて送られたフレームでなければ捨てる
        if !self.should_process(frame_hdr.dst_addr()) {
            return Ok(());
        }

        match frame_hdr.frame_type() {
            link::FrameType::IP => {
                let (_packet_hdr, _rest) = <internet::IP as internet::InternetLayer>::run(
                    &internet::IP(),
                    &raw_ip_packet,
                )?;
            }
            _ => unimplemented!(),
        }

        Ok(())
    }

    // プロトコルスタックが処理すべきパケットかどうか検査
    fn should_process(&self, frame_dst_addr: link::MacAddress) -> bool {
        self.frame_target_is_nic(frame_dst_addr) || frame_dst_addr == link::BLOADCAST_MAC_ADDRESS
    }
    fn frame_target_is_nic(&self, frame_dst_addr: link::MacAddress) -> bool {
        self.device.device_addr() == frame_dst_addr
    }
}
