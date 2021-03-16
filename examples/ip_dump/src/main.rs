use std::{
    env,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use peachps::{interface::tap::TAPDevice, protocol::ip};

const HOST_ADDR: ip::IPv4Addr = ip::IPv4Addr([192, 168, 33, 155]);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interface_name = env::args()
        .nth(1)
        .expect("usage: ./ip_dump <interface_name>");

    match TAPDevice::create(&interface_name) {
        Ok(dev) => {
            ip_dump(dev)?;
        }
        Err(e) => eprintln!("Error found: {}", e),
    }

    Ok(())
}

fn ip_dump(dev: TAPDevice) -> Result<(), Box<dyn std::error::Error>> {
    let mut handles = Vec::new();

    // 受信, 処理スレッドを作る
    let (sender, receiver) = mpsc::channel();
    handles.push(thread::spawn(move || rcv_thread(dev, sender)));
    handles.push(thread::spawn(move || output_thread(receiver)));

    for handle in handles {
        let _ = handle.join().unwrap();
    }

    Ok(())
}

fn output_thread(channel: Receiver<ip::PacketHeader>) {
    loop {
        if let Ok(frame) = channel.recv() {
            eprintln!("{}", frame);
        }
    }
}

fn rcv_thread(dev: TAPDevice, channel: Sender<ip::PacketHeader>) {
    loop {
        match ip::input(dev, HOST_ADDR) {
            Ok((ip_packet, _rest)) => {
                channel.send(ip_packet).unwrap();
            }
            Err(e) => match e {
                ip::IPError::Ignore => {}
                _ => panic!("Error found: {}", e),
            },
        }
    }
}
