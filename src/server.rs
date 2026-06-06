use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::resp::parse::parse_request;
use crate::storage::Storage;

const BUF_SIZE: usize = 512;

pub struct Server {
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
    storage: Arc<Mutex<Storage>>,
}
impl Server {
    pub async fn new(address: &str, max_connections: usize) -> std::io::Result<Self> {
        Ok(Server {
            listener: TcpListener::bind(address).await?,
            limit_connections: Arc::new(Semaphore::new(max_connections)),
            storage: Arc::new(Mutex::new(Storage::new())),
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

            let storage = self.storage.clone();

            let mut check_expiration = tokio::time::interval(Duration::from_millis(10));

            tokio::select! {
                connection = self.listener.accept() => {
                    match connection {
                    Ok((stream, _)) => {
                        println!("Connection accepted");
                        tokio::spawn(async move {
                            handle_connection(stream, storage).await;
                            drop(permit);
                        });
                    }
                    Err(e) => {
                        println!("Error: {}", e);
                        continue;
                    }

                    }
                }
                _ = check_expiration.tick() => {
                    tokio::spawn(expire_keys(self.storage.clone()));
                }

            }
        }
    }
}

async fn expire_keys(storage: Arc<Mutex<Storage>>) {
    let mut storage = storage.lock().unwrap();
    storage.expire_keys();
}

async fn handle_connection(mut stream: TcpStream, db: Arc<Mutex<Storage>>) {
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
