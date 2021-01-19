mod internet;
mod link;
mod network_device;
mod protocol_stack;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: ./peachps [socket/tap] [ip-addr]");
        std::process::exit(1);
    }

    // ProtocolStackが静的ディスパッチによってネットワークデバイスを分けるため，このように．
    // 本当はBox<dyn trait>にしたいが，そうすると実行速度が犠牲になってしまう．
    match args[1].as_str() {
        "socket" => {
            let sock = network_device::setup_raw_socket("eth0".to_string())?;
            let stack = protocol_stack::ProtocolStack::new(sock, link::Ethernet());

            raw_socket_run(stack).await?;
        }
        "tap" => {
            let tap_dev = network_device::setup_tap_device("/dev/net/tun".to_string())?;
            let stack = protocol_stack::ProtocolStack::new(tap_dev, link::Ethernet());

            tap_device_run(stack).await?;
        }
        _ => {
            eprintln!("unsupported such a method => '{}'", args[1]);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Socketの場合
async fn raw_socket_run(
    mut stack: protocol_stack::DefaultPS<network_device::Socket>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        stack.run().await?;
    }
}

/// TapDeviceの場合
async fn tap_device_run(
    mut stack: protocol_stack::DefaultPS<network_device::TapDevice>,
) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        stack.run().await?;
    }
}
