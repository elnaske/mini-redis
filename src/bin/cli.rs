use std::io::Write;
use std::io::prelude::*;
use std::net::TcpStream;
use std::{env, process};

use mini_redis::client::parse_command;
use mini_redis::resp::parse::RESPType;
use mini_redis::resp::parse::parse_response;

const ADDRESS: &str = "127.0.0.1:6379";

fn response_to_string(typ: RESPType) -> String {
    match typ {
        RESPType::SimpleString(s) => s,
        RESPType::BulkString(s) => format!("\"{}\"", s),
        RESPType::NullString => String::from("(nil)"),
        RESPType::Array(arr) => {
            let mut res = Vec::with_capacity(arr.len());
            for t in arr {
                res.push(response_to_string(t));
            }
            res.join("\n")
        }
    }
}

fn repl(addr: &str) {
    let mut stream = TcpStream::connect(addr).unwrap();

    let mut input = String::new();

    loop {
        print!("{}> ", addr);

        std::io::stdout().flush().unwrap();

        input.clear();
        std::io::stdin().read_line(&mut input).unwrap();

        let args = input.trim();

        if args == "exit" {
            break;
        }

        match parse_command(args.split_whitespace()) {
            Ok(cmd) => {
                stream.write_all(cmd.to_resp().as_bytes()).unwrap();
                let mut buffer = [0; 512];
                stream.read(&mut buffer).unwrap();

                let response = parse_response(&buffer).unwrap();

                println!("{}", response_to_string(response));
            }
            Err(e) => {
                println!("Error: {e}");
            }
        }
    }
}

fn main() {
    let mut args = env::args();
    if args.len() == 1 {
        repl(ADDRESS);
    } else {
        args.next();

        let cmd = parse_command(args).unwrap_or_else(|err| {
            eprintln!("{err}");
            process::exit(1);
        });

        let mut stream = TcpStream::connect(ADDRESS).unwrap();
        stream.write_all(cmd.to_resp().as_bytes()).unwrap();
        let mut buffer = [0; 512];
        stream.read(&mut buffer).unwrap();

        let response = parse_response(&buffer).unwrap();

        println!("{}", response_to_string(response));
    }
}
