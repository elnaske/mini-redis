use mini_redis::DEFAULT_ADDRESS;
use mini_redis::server::Server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let server = Server::new(DEFAULT_ADDRESS, 256).await?;

    server.run().await
}
