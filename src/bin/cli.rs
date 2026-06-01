use std::{env, process};
use mini_redis::client::parse_command;
fn main() {
    let cmd = parse_command(env::args()).unwrap_or_else(|err| {
        eprintln!("{err}");
        process::exit(1);
    });
    println!("{:?}", cmd);
}

