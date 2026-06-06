use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::time;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::resp::parse::parse_request;
use crate::storage::Storage;

const BUF_SIZE: usize = 512;

struct ConnectionHandler {
    stream: TcpStream,
    db: Arc<Mutex<Storage>>,
}
impl ConnectionHandler {
    pub async fn run(&mut self) {
        let mut buffer = [0; BUF_SIZE];

        loop {
            match self.stream.read(&mut buffer).await {
                Ok(size) if size > 0 => {
                    let cmd = parse_request(&buffer);

                    let response = match cmd {
                        Ok(cmd) => cmd.execute(&self.db),
                        Err(e) => format!("+{}\r\n", e),
                    };

                    if let Err(e) = self.stream.write_all(response.as_bytes()).await {
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
}

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

    pub async fn run(&mut self) -> std::io::Result<()> {
        loop {
            let permit = self
                .limit_connections
                .clone()
                .acquire_owned()
                .await
                .unwrap();

            let mut check_expiration = tokio::time::interval(Duration::from_millis(10));

            tokio::select! {
                stream = self.accept_connection() => {
                    match stream {
                    Ok(stream) => {
                        println!("Connection accepted");

                        let mut handler = ConnectionHandler {
                            stream, db: self.storage.clone()
                        };

                        tokio::spawn(async move {
                            handler.run().await;
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

    async fn accept_connection(&mut self) -> std::io::Result<TcpStream> {
        let mut backoff = 1;

        loop {
            match self.listener.accept().await {
                Ok((stream, _)) => return Ok(stream),
                Err(err) => {
                    if backoff > 64 {
                        return Err(err.into());
                    }
                }
            }

            time::sleep(Duration::from_secs(backoff)).await;

            backoff *= 2;
        }
    }
}

async fn expire_keys(storage: Arc<Mutex<Storage>>) {
    let mut storage = storage.lock().unwrap();
    storage.expire_keys();
}
