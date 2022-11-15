mod helpers;

use std::sync::mpsc::channel;
use flatbuffers::FlatBufferBuilder;
use rstest::*;
use arque_driver::Client;
use arque_store::{Server, ServerConfig};
use tempdir::TempDir;
use helpers::generate_fake_insert_event_request;

#[rstest]
#[tokio::test]
async fn test_insert_event() {
  let (stop_tx, stop_rx) = channel::<()>();

  std::thread::spawn(move || {
    let temp_dir = TempDir::new("arque_test").unwrap();

    let server = Server::new(ServerConfig {
      data_path: Some(temp_dir.path())
    });

    server.serve("tcp://*:4000".to_string(), stop_rx).unwrap();
  });

  let client = Client::connect("tcp://localhost:4000".to_string()).await.unwrap();

  let mut fbb = FlatBufferBuilder::new();
  let request = generate_fake_insert_event_request(&mut fbb);
  fbb.finish(request, None);

  client.send(fbb.finished_data()).await.unwrap();

  stop_tx.send(()).unwrap();
}

#[rstest]
#[tokio::test]
async fn test_invalid_aggregate_version() {
}
