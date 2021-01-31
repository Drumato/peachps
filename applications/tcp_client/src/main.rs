use std::{env, io::prelude::*};
use std::{net::TcpStream, process};

const CLIENT_MSG: &'static str = "Hello from Drumato!";
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<String>>();
    if args.len() < 3 {
        eprintln!("usage: ./client <ipv4-address> <port>");
        process::exit(1);
    }

    let mut stream = TcpStream::connect(&format!("{}:{}", args[1], args[2]))?;
    let mut seq = 0;

    loop {
        let mut buf = [0; 2048];

        let nbytes = stream.write(&format!("{} (seq={})\n", CLIENT_MSG, seq).as_bytes())?;
        seq += 1;
        eprintln!("send {} bytes to server", nbytes);

        let nbytes = stream.read(&mut buf)?;
        if nbytes == 0 {
            break;
        }

        let msg = std::str::from_utf8(&buf)?;
        eprintln!("'{}' from server", msg.replace("\n", ""));
    }

    Ok(())
}
