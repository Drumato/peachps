mod link;
mod network_device;

use network_device::NetworkDevice;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: ./peachps [socket/tap]");
        std::process::exit(1);
    }
    match args[1].as_str() {
        "socket" => {
            let mut sock = network_device::setup_raw_socket("eth0".to_string())?;
            eprintln!("mac address => {}", sock.mac_addr);

            loop {
                let mut buf: Vec<u8> = Vec::with_capacity(2048);
                let nbytes = sock.read(buf.as_mut_slice()).await?;
                eprintln!("reading {} bytes", nbytes);

                if nbytes == 0 {
                    break;
                }
            }
        }
        "tap" => {
            let mut tap_dev = network_device::setup_tap_device("/dev/net/tun".to_string())?;
            eprintln!("mac address => {}", tap_dev.mac_addr);

            loop {
                let mut buf: Vec<u8> = Vec::with_capacity(2048);
                let nbytes = tap_dev.read(buf.as_mut_slice()).await?;
                eprintln!("reading {} bytes", nbytes);

                if nbytes == 0 {
                    break;
                }
            }
        }
        _ => {
            eprintln!("unsupported such a method => '{}'", args[1]);
            std::process::exit(1);
        }
    };

    Ok(())
}
