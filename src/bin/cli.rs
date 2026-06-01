use std::io::Write;
use std::io::prelude::*;
use std::net::TcpStream;
use std::{env, process};

use mini_redis::client::parse_command;

const ADDRESS: &str = "127.0.0.1:6379";

fn main() {
    let cmd = parse_command(env::args()).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });

    let mut stream = TcpStream::connect(ADDRESS).unwrap();
    stream.write_all(cmd.to_resp().as_bytes()).unwrap();
    let mut buffer = [0; 512];
    stream.read_exact(&mut buffer).unwrap();
    println!("{}", String::from_utf8_lossy(&buffer));
}
