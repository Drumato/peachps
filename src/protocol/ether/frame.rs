pub struct Frame {
    dst_addr: MacAddress,
    src_addr: MacAddress,
    length: u16,
}

pub struct MacAddress([u8; 6]);
