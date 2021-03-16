use thiserror::Error;

use crate::protocol::ether;

#[derive(Error, Debug)]
pub enum IPError {
    #[error("failed to parse bytes as packet")]
    ParseBytesAsPacket,
    #[error("ignore this packet")]
    Ignore,
    #[error("{err:}")]
    EthernetError { err: ether::EthernetError },
}
