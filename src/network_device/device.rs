use crate::link::MacAddress;
use async_trait::async_trait;
use thiserror::Error;

#[async_trait]
pub trait NetworkDevice: Copy + Send + Sync {
    /// デバイスからデータを読み込む
    async fn read(&self, buf: &mut [u8]) -> Result<usize, NetworkDeviceError>;
    async fn write(&self, buf: &[u8]) -> Result<usize, NetworkDeviceError>;

    /// イーサネットフレームのdst_addrが自身に向いているかチェックするために使用
    fn device_addr(&self) -> MacAddress;
}

#[derive(Error, Debug, Clone, Copy)]
pub enum NetworkDeviceError {
    #[error("malformed device name")]
    MalformedDeviceName,
    #[error("malformed interface name")]
    MalformedInterfaceName,
    #[error("failed to setup network device")]
    FailedToSetupNetworkDevice,
    #[error("failed to read bytes stream from {fd:}")]
    FailedToReadFrom { fd: FileDescriptor },
    #[error("failed to write bytes stream to {fd:}")]
    FailedToWriteTo { fd: FileDescriptor },
    #[error("time out")]
    Timeout,
}

pub type FileDescriptor = libc::c_int;
