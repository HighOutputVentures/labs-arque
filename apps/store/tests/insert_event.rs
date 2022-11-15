use std::sync::mpsc::channel;
use rstest::*;
use arque_driver::Client;
use arque_store::{Server, ServerConfig};
use tempdir::TempDir;

#[rstest]
#[tokio::test]
async fn test_insert_event() {
  let (shutdown_tx, shutdown_rx) = channel::<()>();

  std::thread::spawn(move || {
    let temp_dir = TempDir::new("arque").unwrap();

    let server = Server::new(ServerConfig {
      data_path: Some(temp_dir.path())
    });

    server.serve("tcp://*:4000".to_string(), shutdown_rx).unwrap();
  });

  let client = Client::connect("tcp://localhost:4000".to_string()).await.unwrap();

  client.send("hello world".as_bytes()).await.unwrap();

  shutdown_tx.send(()).unwrap();
}

#[rstest]
#[tokio::test]
async fn test_invalid_aggregate_version() {
}
