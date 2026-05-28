use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::parse::parse_input;

const BUF_SIZE: usize = 512;

pub type DB = Arc<Mutex<HashMap<String, String>>>;

pub struct Server {
    listener: TcpListener,
    db: DB,
}
impl Server {
    pub async fn init(address: &str) -> std::io::Result<Self> {
        Ok(Server {
            listener: TcpListener::bind(address).await?,
            db: Arc::new(Mutex::new(HashMap::<String, String>::new())),
        })
    }

    pub async fn run(&self) -> std::io::Result<()> {
        loop {
            match self.listener.accept().await {
                Ok((stream, _)) => {
                    println!("Connection accepted");
                    tokio::spawn(handle_connection(stream, self.db.clone()));
                }
                Err(e) => {
                    println!("Error: {}", e);
                    continue;
                }
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream, db: DB) {
    let mut buffer = [0; BUF_SIZE];

    loop {
        match stream.read(&mut buffer).await {
            Ok(size) if size > 0 => {
                // println!("Received: {:?}", buffer);

                let cmd = parse_input(&buffer);

                let response = match cmd {
                    Ok(cmd) => cmd.execute(&db),
                    Err(e) => format!("+{}\r\n", e),
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
