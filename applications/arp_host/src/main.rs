use peachps::{internet, link, network_device, option};
use std::collections::HashSet;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: ./arp_host <interface_name> (debug)");
        std::process::exit(1);
    }

    let mut sock = network_device::setup_raw_socket(args[1].clone())?;
    let ip_layer = {
        let mut s = HashSet::new();
        s.insert(internet::InternetProtocol::ARP);
        s
    };
    let tcp_layer = Default::default();

    let opt: option::PeachPSOption = option::PeachPSOption::from_yaml("config.yaml");

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
