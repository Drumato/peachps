use crate::link;

pub trait LinkLayer {
    type FrameHeader: Frame;
    fn run(&self, buf: &[u8]) -> Result<(Self::FrameHeader, Vec<u8>), Box<dyn std::error::Error>>;
}

pub trait Frame: std::fmt::Display {
    fn dst_addr(&self) -> link::MacAddress;
    fn frame_type(&self) -> link::FrameType;
}
