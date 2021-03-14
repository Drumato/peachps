use std::{
    collections::VecDeque,
    env,
    sync::{Arc, Mutex},
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
    let queue = Arc::new(Mutex::new(VecDeque::new()));

    // 受信, 処理スレッドを作る
    let queue1 = Arc::clone(&queue);
    handles.push(thread::spawn(move || rcv_thread(dev, queue)));
    handles.push(thread::spawn(move || output_thread(queue1)));

    for handle in handles {
        let _ = handle.join().unwrap();
    }

    Ok(())
}

fn output_thread(queue: Arc<Mutex<VecDeque<ether::Frame>>>) {
    loop {
        if let Ok(ref mut q) = queue.lock() {
            if let Some(frame) = q.pop_front() {
                eprintln!("{}", frame);
            }
        }
    }
}

fn rcv_thread(dev: TAPDevice, queue: Arc<Mutex<VecDeque<ether::Frame>>>) {
    loop {
        let mut buffer = [0; 1024];

        let nbytes = dev.read(&mut buffer).unwrap();

        if nbytes == 0 {
            continue;
        }

        if let Ok(ref mut q) = queue.lock() {
            let ether_frame = ether::Frame::from_bytes(&buffer).unwrap();
            q.push_back(ether_frame);
        }
    }
}
