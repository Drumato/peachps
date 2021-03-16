use std::{
    env,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use peachps::{interface::tap::TAPDevice, protocol::ether};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let interface_name = env::args()
        .nth(1)
        .expect("usage: ./ether_dump <interface_name>");

    match TAPDevice::create(&interface_name) {
        Ok(dev) => {
            ether_dump(dev)?;
        }
        Err(e) => eprintln!("Error found: {}", e),
    }

    Ok(())
}

fn ether_dump(dev: TAPDevice) -> Result<(), Box<dyn std::error::Error>> {
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

fn output_thread(channel: Receiver<ether::Frame>) {
    loop {
        if let Ok(frame) = channel.recv() {
            eprintln!("{}", frame);
        }
    }
}

fn rcv_thread(dev: TAPDevice, channel: Sender<ether::Frame>) {
    loop {
        match ether::input(dev) {
            Ok((ether_frame, _rest)) => {
                channel.send(ether_frame).unwrap();
            }
            Err(e) => match e {
                ether::EthernetError::EOF | ether::EthernetError::Ignore => {
                    continue;
                }
                _ => panic!("Error found: {}", e),
            },
        }
    }
}
