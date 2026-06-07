# mini-redis

A minimal implementation of a Redis server and client.
I mainly wrote it to get to grips with Tokio and async Rust in general.
While this project is nowhere near production-grade, it does support the following:

- Concurrent server and client with connection limits
- Client CLI with REPL
- Four commands: `PING`, `ECHO`, `SET`, and `GET`
- Key expiry

## Usage
To run the server using cargo:
```bash
cargo run --bin mini-redis-server
```

The server will bind to port 6379, which is the default for Redis.
This means that if you have Redis installed on your system, you may already have a server running on the same port.
In that case, you will have to shut down the existing server before running `mini-redis-server`.

To run the CLI client:
```bash
cargo run --bin mini-redis-cli
```

This will launch an interactive REPL session.
You can also send a command directly like so:
```bash
cargo run --bin mini-redis-cli <command> <args>

# E.g.
cargo run --bin mini-redis-cli echo "hello world"
```

## Acknowledgements

This project was mainly inspired by Tokio's [mini-redis](https://github.com/tokio-rs/mini-redis) repository.
While the [official Tokio tutorial](https://tokio.rs/tokio/tutorial) is decent, reading through the mini-redis code gave me a much better understanding of the actual practical applications of async.
I also frequently used it to reference implementation details.
Another resource I followed was [Rust Projects - Writing a Redis Clone](https://rust-projects-write-a-redis-clone.github.io/), though I did end up deviating quite a bit from it.
Lastly, the [Redis docs](https://redis.io/docs/latest/) were my main reference for implementing RESP.