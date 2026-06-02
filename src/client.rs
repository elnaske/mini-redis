use std::io::prelude::*;
use std::io::{self, Write};
use std::net::TcpStream;

use crate::commands::{Command, Echo, Get, Ping, Set};
use crate::resp::parse::{RESPType, parse_response};

pub struct Client {
    address: String,
    stream: TcpStream,
}
impl Client {
    pub fn new(address: &str) -> std::io::Result<Self> {
        Ok(Client {
            address: address.to_owned(),
            stream: TcpStream::connect(address)?,
        })
    }

    pub fn repl(&mut self) {
        let mut input = String::new();

        loop {
            print!("{}> ", self.address);

            io::stdout().flush().unwrap();

            input.clear();
            io::stdin().read_line(&mut input).unwrap();

            let args = input.trim();

            if args == "exit" {
                break;
            }

            match self.process_command(args.split_whitespace()) {
                Ok(response) => println!("{}", response),
                Err(e) => println!("Error: {e}"),
            }
        }
    }

    pub fn process_command<S>(&mut self, args: impl Iterator<Item = S>) -> Result<String, String>
    where
        S: AsRef<str>,
    {
        let cmd = parse_command(args)?;
        self.send_request(cmd);
        let response = self.get_response();
        Ok(response)
    }

    fn send_request(&mut self, cmd: Command) {
        self.stream.write_all(cmd.to_resp().as_bytes()).unwrap();
    }

    fn get_response(&mut self) -> String {
        let mut buffer = [0; 512];
        self.stream.read(&mut buffer).unwrap();

        let response = parse_response(&buffer).unwrap();
        response_to_string(response)
    }
}

pub fn parse_command<S>(mut args: impl Iterator<Item = S>) -> Result<Command, String>
where
    S: AsRef<str>,
{
    match args.next() {
        Some(cmd) => match &cmd.as_ref().to_lowercase()[..] {
            "ping" => Ok(Command::Ping(Ping::new())),
            "echo" => {
                let Some(msg) = args.next() else {
                    return Err(String::from("Usage: client echo <message>"));
                };
                Ok(Command::Echo(Echo::new(msg.as_ref().to_owned())))
            }
            "set" => {
                let Some(key) = args.next() else {
                    return Err(String::from("Usage: client set <key> <value>"));
                };
                let Some(value) = args.next() else {
                    return Err(String::from("Usage: client set <key> <value>"));
                };

                Ok(Command::Set(Set::new(
                    key.as_ref().to_owned(),
                    value.as_ref().to_owned(),
                )))
            }
            "get" => {
                let Some(key) = args.next() else {
                    return Err(String::from("Usage: client get <key>"));
                };

                Ok(Command::Get(Get::new(key.as_ref().to_owned())))
            }
            other => Err(format!("Command `{}` not implemented", other)),
        },
        None => Err(String::from("Usage: client <command> <args>")),
    }
}

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
