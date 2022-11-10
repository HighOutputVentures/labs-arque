use std::sync::mpsc::channel;
use rstest::*;
use arque_driver::Client;
use arque_store::Server;

#[rstest]
#[tokio::test]
async fn test_insert_event() {
  let (tx, rx) = channel::<()>();

  std::thread::spawn(move || {
    Server::serve("tcp://*:4000".to_string(), |req| {
      println!("request: {:?}", req);
  
      vec![]
    }, rx).unwrap();
  });

  let client = Client::connect("tcp://localhost:4000".to_string()).await.unwrap();

  client.send("hello world".as_bytes()).await.unwrap();

  tx.send(()).unwrap();
}

#[rstest]
#[tokio::test]
async fn test_invalid_aggregate_version() {
}
