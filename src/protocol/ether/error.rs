use thiserror::Error;

#[derive(Debug, Error)]
pub enum EthernetError {
    #[error("failed to parse bytes as ethernet frame")]
    ParseBytesAsFrame,
    #[error("eof")]
    EOF,
    #[error("failed to read from network device")]
    Read,
    #[error("ignore this frame")]
    Ignore,
}
