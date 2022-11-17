mod helpers;

use arque_common::request_generated::{
    InsertEventRequestBody, InsertEventRequestBodyArgs, Request, RequestArgs, RequestBody,
};
use arque_driver::Client;
use arque_store::{InsertEventError, InsertEventParams, RocksDBStore, Server, ServerConfig, Store};
use flatbuffers::FlatBufferBuilder;
use get_port::{tcp::TcpPort, Ops};
use helpers::{generate_fake_event, generate_fake_insert_event_request, GenerateFakeEventArgs};
use rstest::*;
use std::sync::mpsc::channel;
use tempdir::TempDir;
use uuid::Uuid;

#[rstest]
#[tokio::test]
async fn test_insert_event() {
    let (stop_tx, stop_rx) = channel::<()>();

    let tcp_port = TcpPort::any("127.0.0.1").unwrap();

    std::thread::spawn(move || {
        let temp_dir = TempDir::new("arque_test").unwrap();

        let mut server_endpoint = String::from("tcp://*:");
        server_endpoint.push_str(&tcp_port.to_string());

        let server = Server::new(ServerConfig {
            data_path: Some(temp_dir.path()),
        });

        server.serve(server_endpoint, stop_rx).unwrap();
    });

    let mut client_endpoint = String::from("tcp://localhost:");
    client_endpoint.push_str(&tcp_port.to_string());

    let client = Client::connect(client_endpoint).await.unwrap();

    let mut fbb = FlatBufferBuilder::new();
    let request = generate_fake_insert_event_request(&mut fbb);
    fbb.finish(request, None);

    client.send(fbb.finished_data()).await.unwrap();

    stop_tx.send(()).unwrap();
}

#[rstest]
#[tokio::test]
async fn test_invalid_aggregate_version() {
    let (stop_tx, stop_rx) = channel::<()>();

    let tcp_port = TcpPort::any("127.0.0.1").unwrap();

    let temp_dir = TempDir::new("arque_test").unwrap();

    let store = RocksDBStore::open(temp_dir.path()).unwrap();

    let mut input_fbb = FlatBufferBuilder::new();

    let args = GenerateFakeEventArgs::default();

    let event = generate_fake_event(&mut input_fbb, &args);

    input_fbb.finish(event, None);

    let aggregate_id = Uuid::new_v4();
    let id = Uuid::new_v4();

    let params = InsertEventParams {
        aggregate_id: aggregate_id.as_bytes(),
        id: id.as_bytes(),
        payload: &input_fbb.finished_data().to_vec(),
        aggregate_version: 1,
    };

    store.insert_event(params).unwrap();

    std::thread::spawn(move || {
        let temp_dir = TempDir::new("arque_test").unwrap();

        let mut server_endpoint = String::from("tcp://*:");
        server_endpoint.push_str(&tcp_port.to_string());

        let server = Server::new(ServerConfig {
            data_path: Some(temp_dir.path()),
        });

        server.serve(server_endpoint, stop_rx).unwrap();
    });

    let mut client_endpoint = String::from("tcp://localhost:");
    client_endpoint.push_str(&tcp_port.to_string());

    let client = Client::connect(client_endpoint).await.unwrap();

    let mut fbb = FlatBufferBuilder::new();

    let event = generate_fake_event(&mut fbb, &args);

    let args = InsertEventRequestBodyArgs { event: Some(event) };

    let body = InsertEventRequestBody::create(&mut fbb, &args);

    let args = RequestArgs {
        body: Some(body.as_union_value()),
        body_type: RequestBody::InsertEvent,
    };

    let request = Request::create(&mut fbb, &args);

    fbb.finish(request, None);

    let e = client.send(fbb.finished_data()).await.unwrap_err();

    println!("error: {:?}", e);

    stop_tx.send(()).unwrap();
}
