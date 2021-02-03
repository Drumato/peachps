use peachps::{link, network_device, option};
pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: ./icmp_pong <interface_name>");
        std::process::exit(1);
    }

    let sock = network_device::setup_raw_socket(args[1].clone())?;

    let opt: option::PeachPSOption = option::PeachPSOption::from_yaml("config.yaml");

    peachps::run(opt, sock, link::LinkProtocol::Ethernet)?;

    Ok(())
}
