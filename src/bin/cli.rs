use std::{env, process};

use mini_redis::client::Client;

const ADDRESS: &str = "127.0.0.1:6379";

fn main() {
    let mut client = Client::new(ADDRESS).unwrap();
    let mut args = env::args();

    if args.len() == 1 {
        client.repl();
    } else {
        args.next();

        match client.process_command(args) {
            Ok(response) => println!("{}", response),
            Err(e) => {
                eprintln!("{e}");
                process::exit(1)
            }
        }
    }
}
