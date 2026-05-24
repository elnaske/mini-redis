use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const BUF_SIZE: usize = 512;

fn handle_connection(stream: &mut TcpStream) {
    let mut buffer = [0; BUF_SIZE];

    loop {
        match stream.read(&mut buffer) {
            Ok(size) if size > 0 => {
                // println!("Received: {:?}", buffer);
                
                let response = "+PONG\r\n";
                stream.write(response.as_bytes()).unwrap();

                stream.flush().unwrap();
            }
            Ok(_) => {
                println!("Connection closed");
                break;
            }
            Err(e) => {
                println!("Error: {e}");
            }
        }

    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                handle_connection(&mut stream);
            }
            Err(e) => println!("Error: {}", e),
        }
    }
}
