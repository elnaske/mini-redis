pub mod server;
use server::Server;

pub mod commands;
pub mod parse;

const ADDRESS: &str = "127.0.0.1:6379";

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let server = Server::init(ADDRESS).await?;

    server.run().await
}
