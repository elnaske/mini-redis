use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::resp::parse::parse_request;

const BUF_SIZE: usize = 512;

pub type DB = Arc<Mutex<HashMap<String, String>>>;

pub struct Server {
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
    db: DB,
}
impl Server {
    pub async fn new(address: &str, max_connections: usize) -> std::io::Result<Self> {
        Ok(Server {
            listener: TcpListener::bind(address).await?,
            limit_connections: Arc::new(Semaphore::new(max_connections)),
            db: Arc::new(Mutex::new(HashMap::<String, String>::new())),
        })
    }

    pub async fn run(&self) -> std::io::Result<()> {
        loop {
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();

            let db = self.db.clone();

            match self.listener.accept().await {
                Ok((stream, _)) => {
                    println!("Connection accepted");
                    tokio::spawn(async move {
                        handle_connection(stream, db).await;
                        drop(permit);
                    });
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

                let cmd = parse_request(&buffer);

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
