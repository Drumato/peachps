use peachps::{internet, link, network_device, transport};
use std::collections::HashSet;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("usage: ./tcp_echo <interface_name>");
        std::process::exit(1);
    }

    let mut sock = network_device::setup_raw_socket(args[1].clone())?;
    let ip_layer = {
        let mut s = HashSet::new();
        s.insert(internet::InternetProtocol::IP);
        s
    };
    let tcp_layer = {
        let mut s = HashSet::new();
        s.insert(transport::TransportProtocol::TCP);
        s
    };

    loop {
        peachps::rx_transport(
            &mut sock,
            link::LinkProtocol::Ethernet,
            &ip_layer,
            &tcp_layer,
        )
        .await;
    }

    Ok(())
}
