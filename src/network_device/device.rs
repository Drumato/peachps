use crate::link::MacAddress;
use async_trait::async_trait;
use thiserror::Error;
#[async_trait]
pub trait NetworkDevice {
    /// デバイスからデータを読み込む
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, NetworkDeviceError>;

    /// イーサネットフレームのdst_addrが自身に向いているかチェックするために使用
    fn device_addr(&self) -> MacAddress;
}

#[derive(Error, Debug)]
pub enum NetworkDeviceError {
    #[error("malformed device name")]
    MalformedDeviceName,
    #[error("malformed interface name")]
    MalformedInterfaceName,
    #[error("failed to setup network device")]
    FailedToSetupNetworkDevice,
    #[error("failed to read byte stream from {fd:}")]
    FailedToReadFrom { fd: FileDescriptor },
}

pub type FileDescriptor = libc::c_int;
