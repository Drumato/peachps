pub trait InternetLayer {
    type PacketHeader;
    fn run(&self, buf: &[u8]) -> Result<(Self::PacketHeader, Vec<u8>), Box<dyn std::error::Error>>;
}
