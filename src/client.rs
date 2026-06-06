use std::io::{self, Write};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::RecvError;

use crate::commands::{Command, Echo, Get, Ping, Set};
use crate::resp::parse::{RESPType, parse_response};
use crate::storage::KeyExpiry;

pub struct Message {
    cmd: Command,
    responder: oneshot::Sender<String>,
}
impl Message {
    pub fn new(cmd: Command, responder: oneshot::Sender<String>) -> Self {
        Message { cmd, responder }
    }
}

pub async fn request_client(cmd: Command, tx: mpsc::Sender<Message>) -> Result<String, RecvError> {
    let (tx_resp, rx_resp) = oneshot::channel();

    tx.send(Message::new(cmd, tx_resp)).await.unwrap();

    rx_resp.await
}

pub struct Client {
    address: String,
    stream: TcpStream,
}
impl Client {
    pub async fn connect(address: &str) -> std::io::Result<Self> {
        Ok(Client {
            address: address.to_owned(),
            stream: TcpStream::connect(address).await?,
        })
    }

    pub async fn repl(&mut self) {
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

            match self.process_command(args.split_whitespace()).await {
                Ok(response) => println!("{}", response),
                Err(e) => println!("Error: {e}"),
            }
        }
    }

    pub async fn process_command<S>(
        &mut self,
        args: impl Iterator<Item = S>,
    ) -> Result<String, String>
    where
        S: AsRef<str>,
    {
        let cmd = parse_command(args)?;
        self.send_request(cmd).await.map_err(|e| e.to_string())?;
        let response = self.get_response().await;
        Ok(response)
    }

    pub async fn send_request(&mut self, cmd: Command) -> std::io::Result<()> {
        self.stream.write_all(cmd.to_resp().as_bytes()).await?;
        Ok(())
    }

    pub async fn get_response(&mut self) -> String {
        let mut buffer = [0; 512];
        self.stream.read(&mut buffer).await.unwrap(); // TODO: check size of read

        let response = parse_response(&buffer).unwrap();
        response_to_string(response)
    }

    pub async fn manage_requests(&mut self, mut rx: mpsc::Receiver<Message>) {
        while let Some(msg) = rx.recv().await {
            self.send_request(msg.cmd).await.unwrap();
            let response = self.get_response().await;

            msg.responder.send(response).unwrap();
        }
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

                let expire = match args.next() {
                    Some(arg) => match &arg.as_ref().to_lowercase()[..] {
                        "ex" => match args.next() {
                            Some(t) => {
                                let t = t
                                    .as_ref()
                                    .parse::<u64>()
                                    .map_err(|e| e.to_string())
                                    .unwrap();
                                Some(KeyExpiry::EX(t))
                            }
                            None => {
                                return Err(format!("Expected argument after {}", arg.as_ref()));
                            }
                        },
                        "px" => match args.next() {
                            Some(t) => {
                                let t = t
                                    .as_ref()
                                    .parse::<u64>()
                                    .map_err(|e| e.to_string())
                                    .unwrap();
                                Some(KeyExpiry::PX(t))
                            }
                            None => {
                                return Err(format!("Expected argument after {}", arg.as_ref()));
                            }
                        },
                        _ => return Err(format!("Unknown argument: {}", arg.as_ref())),
                    },
                    None => None,
                };

                Ok(Command::Set(Set::new(
                    key.as_ref().to_owned(),
                    value.as_ref().to_owned(),
                    expire,
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
