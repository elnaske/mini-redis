use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Semaphore, broadcast, mpsc};
use tokio::time;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::resp::parse::parse_request;
use crate::storage::Storage;

const BUF_SIZE: usize = 512;

struct ConnectionHandler {
    stream: TcpStream,
    shutdown_notice: broadcast::Receiver<()>,
    _shutdown_complete: mpsc::Sender<()>,
    db: Arc<Mutex<Storage>>,
    should_shutdown: bool,
}
impl ConnectionHandler {
    pub fn new(
        stream: TcpStream,
        shutdown_notice: broadcast::Receiver<()>,
        shutdown_complete: mpsc::Sender<()>,
        db: Arc<Mutex<Storage>>,
    ) -> Self {
        ConnectionHandler {
            stream,
            shutdown_notice,
            _shutdown_complete: shutdown_complete,
            db,
            should_shutdown: false,
        }
    }
    pub async fn run(&mut self) {
        let mut buffer = [0; BUF_SIZE];

        while !self.should_shutdown {
            tokio::select! {
                read = self.stream.read(&mut buffer) => {
                    match read {
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
                _ = self.shutdown_notice.recv() => {
                    self.should_shutdown = true;
                    break;
                }
            }
        }

        println!("Connection shut down");
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

    pub async fn run(&mut self, shutdown_signal: impl Future) {
        let (shutdown_tx, _) = broadcast::channel(1);
        let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

        let mut check_expiration = tokio::time::interval(Duration::from_millis(10));
        tokio::pin!(shutdown_signal);

        loop {
            tokio::select! {
                stream = self.accept_connection() => {
                    match stream {
                    Ok(stream) => {
                        let permit = self
                            .limit_connections
                            .clone()
                            .acquire_owned()
                            .await
                            .unwrap();

                        println!("Connection accepted");

                        let db = self.storage.clone();
                        let shutdown_notice = shutdown_tx.subscribe();

                        let mut handler = ConnectionHandler::new(stream, shutdown_notice, shutdown_complete_tx.clone(), db);

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
                    expire_keys(self.storage.clone()).await;
                }
                _ = &mut shutdown_signal => {
                    println!("Shutting down");
                    break;
                }

            }
        }

        drop(shutdown_tx);
        drop(shutdown_complete_tx);

        let _ = shutdown_complete_rx.recv().await;
        println!("All connections closed; Shutdown complete")
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
