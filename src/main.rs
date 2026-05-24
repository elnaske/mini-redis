use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub mod parse;
use parse::parse_input;

const ADDRESS: &str = "127.0.0.1:6379";
const BUF_SIZE: usize = 512;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind(ADDRESS).await?;

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
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
                // println!("Received: {:?}", buffer);

                let response = "+PONG\r\n";

                if let Err(e) = stream.write_all(response.as_bytes()).await {
                    eprintln!("Error writing to socket: {e}");
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
