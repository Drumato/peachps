pub const DATA_OFFSET_MASK: u8 = 0xf0;

pub struct SegmentHeader {
    src_port: u16,
    dst_port: u16,
    sequence: u32,
    acknowledgement: u32,
    offset: u8,
    control_flag: u8,
    window_size: u16,
    checksum: u16,
    urgent_pointer: u16,
}

impl SegmentHeader {
    pub const LEAST_LENGTH: usize = 20;

    pub fn data_offset(&self) -> u8 {
        (self.offset & DATA_OFFSET_MASK) >> 4
    }
}
