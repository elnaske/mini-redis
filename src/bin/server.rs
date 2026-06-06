use mini_redis::DEFAULT_ADDRESS;
use mini_redis::server::Server;

#[tokio::main]
async fn main() {
    let mut server = Server::new(DEFAULT_ADDRESS, 256).await.unwrap();

    let shutdown_signal = tokio::signal::ctrl_c();
    server.run(shutdown_signal).await
}
