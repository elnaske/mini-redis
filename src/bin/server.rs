use mini_redis::server::Server;

const ADDRESS: &str = "127.0.0.1:6379";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let server = Server::new(ADDRESS).await?;

    server.run().await
}
