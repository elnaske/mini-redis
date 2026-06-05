use tokio::sync::mpsc;

use mini_redis::DEFAULT_ADDRESS;
use mini_redis::client::{Client, Message, request_client};
use mini_redis::commands::{Command, Echo, Ping};

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel::<Message>(32);
    let tx2 = tx.clone();

    let manager = tokio::spawn(async move {
        let mut client = Client::connect(DEFAULT_ADDRESS).await.unwrap();
        client.manage_requests(rx).await;
    });

    let t1 = tokio::spawn(async move {
        let cmd = Command::Ping(Ping::new());
        let res = request_client(cmd, tx).await;
        println!("T1: {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let cmd = Command::Echo(Echo::new(String::from("hello")));
        let res = request_client(cmd, tx2).await;
        println!("T2: {:?}", res);
    });

    manager.await.unwrap();
    t1.await.unwrap();
    t2.await.unwrap();
}
