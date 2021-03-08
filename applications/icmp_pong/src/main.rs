use peachps::{link, network_device, option};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: ./icmp_pong <interface_name>");
        std::process::exit(1);
    }

    let sock = network_device::setup_raw_socket(args[1].clone())?;

    let opt: option::PeachPSOption = option::PeachPSOption::from_yaml("config.yaml");

    eprintln!("MAC: {}", opt.dev_addr);
    eprintln!("IP: {}", opt.ip_addr);

    let items = peachps::Items::new(opt, sock);

    peachps::run(&items, link::LinkProtocol::Ethernet).await?;

    Ok(())
}
