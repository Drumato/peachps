use crate::{byteorder_wrapper, internet, link};

#[allow(dead_code)]
/// Ethernet Frame Header
pub struct FrameHeader {
    pub dst_addr: link::MacAddress,
    pub src_addr: link::MacAddress,
    pub ty: internet::InternetProtocol,
}

impl std::fmt::Display for FrameHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "dst_addr: {}", self.dst_addr)?;
        writeln!(f, "src_addr: {}", self.src_addr)?;
        writeln!(f, "frame_type: {}", self.ty)?;

        Ok(())
    }
}

impl Default for FrameHeader {
    fn default() -> Self {
        Self {
            dst_addr: link::MacAddress([0; 6]),
            src_addr: link::MacAddress([0; 6]),
            ty: internet::InternetProtocol::IP,
        }
    }
}

impl FrameHeader {
    pub const LENGTH: usize = 0xe;

    pub fn to_bytes<E>(&self, err: E) -> Result<Vec<u8>, E>
    where
        E: std::error::Error + Copy,
    {
        let mut buf = Vec::new();
        byteorder_wrapper::write_u48_as_be(&mut buf, self.dst_addr.into(), err)?;
        byteorder_wrapper::write_u48_as_be(&mut buf, self.src_addr.into(), err)?;
        Ok(buf)
    }
}
