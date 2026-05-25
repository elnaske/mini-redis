use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub mod parse;
use parse::parse_input;

use crate::parse::Command;

const ADDRESS: &str = "127.0.0.1:6379";
const BUF_SIZE: usize = 512;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(ADDRESS).await?;

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                println!("Connection accepted");
                tokio::spawn(handle_connection(stream));
            }
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; BUF_SIZE];

    loop {
        match stream.read(&mut buffer).await {
            Ok(size) if size > 0 => {
                println!("Received: {:?}", buffer);

                let cmd = parse_input(&buffer);

                let response = {
                    match cmd {
                        Ok(Command::Ping) => "+PONG\r\n",
                        Ok(Command::Echo(s)) => &format!("${}\r\n{}\r\n", s.len(), s),
                        Err(e) => {
                            println!("Error: {e:?}");
                            "+Error: Invalid Command\r\n"
                        }
                    }
                };

                if let Err(e) = stream.write_all(response.as_bytes()).await {
                    println!("Error writing to socket: {e}");
                }
            }
            Ok(_) => {
                println!("Connection closed");
                break;
            }
            Err(e) => {
                println!("Error: {e}");
                break;
            }
        }
    }
}
