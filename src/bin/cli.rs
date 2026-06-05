use std::{env, process};

use mini_redis::DEFAULT_ADDRESS;
use mini_redis::client::Client;

#[tokio::main]
async fn main() {
    let mut client = Client::connect(DEFAULT_ADDRESS).await.unwrap();
    let mut args = env::args();

    if args.len() == 1 {
        client.repl().await;
    } else {
        args.next();

        match client.process_command(args).await {
            Ok(response) => println!("{}", response),
            Err(e) => {
                eprintln!("{e}");
                process::exit(1)
            }
        }
    }
}
