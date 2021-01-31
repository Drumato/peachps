use peachps::{internet, link, network_device, option, transport};
use std::collections::HashSet;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: ./tcp_echo <interface_name> (debug)");
        std::process::exit(1);
    }

    let mut sock = network_device::setup_raw_socket(args[1].clone())?;
    let ip_layer = {
        let mut s = HashSet::new();
        s.insert(internet::InternetProtocol::IP);
        s.insert(internet::InternetProtocol::ARP);
        s
    };
    let tcp_layer = {
        let mut s = HashSet::new();
        s.insert(transport::TransportProtocol::ICMP);
        s
    };
    let opt: option::PeachPSOption = {
        let mut o: option::PeachPSOption = Default::default();
        o.ip_addr = internet::ip::IPv4Addr::from([192, 168, 111, 240]);
        o.network_mask = internet::ip::IPv4Addr::from([255, 255, 255, 0]);
        o.debug = args.len() == 3 && args[2] == "debug";
        o
    };

    peachps::run(
        opt,
        &mut sock,
        link::LinkProtocol::Ethernet,
        &ip_layer,
        &tcp_layer,
    )
    .await?;

    Ok(())
}
