mod link;
mod tap_device;
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tap_dev = tap_device::setup_tap_device("/dev/net/tun".to_string(), "tap0".to_string())?;
    eprintln!("mac_addr => {:?}", tap_dev.mac_addr);
    Ok(())
}
