mod helpers;

use arque_common::{
    request_generated::{
        InsertEventRequestBody, InsertEventRequestBodyArgs, Request, RequestArgs, RequestBody,
    },
    response_generated::{root_as_response, ResponseStatus},
};
use arque_driver::Client;
use arque_store::{InsertEventParams, RocksDBStore, Server, ServerConfig, Store};
use flatbuffers::FlatBufferBuilder;
use get_port::{tcp::TcpPort, Ops};
use helpers::{
    generate_fake_event, generate_fake_insert_event_request, random_bytes, GenerateFakeEventArgs,
};
use rocksdb::Options;
use rstest::*;
use std::sync::mpsc::channel;
use tempdir::TempDir;

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

    let response_data = client.send(fbb.finished_data()).await.unwrap();

    let response = root_as_response(&response_data).unwrap();

    assert_eq!(
        response.status(),
        ResponseStatus::Ok,
        "should return status Ok"
    );

    stop_tx.send(()).unwrap();
}

#[rstest]
#[tokio::test]
async fn test_invalid_aggregate_version() {
    let (stop_tx, stop_rx) = channel::<()>();

    let tcp_port = TcpPort::any("127.0.0.1").unwrap();

    let temp_dir = TempDir::new("arque_test").unwrap();

    let mut input_fbb = FlatBufferBuilder::new();

    let args = GenerateFakeEventArgs {
        id: Some(random_bytes(12)),
        type_: Some(fastrand::u16(..)),
        aggregate_id: Some(random_bytes(12)),
        aggregate_version: Some(fastrand::u32(..)),
        body: Some(random_bytes(1024)),
        meta: Some(random_bytes(64)),
    };
    let temp = args.clone();

    let event = generate_fake_event(&mut input_fbb, &temp);

    println!("input_event: {:?}", event.to_be_bytes());

    input_fbb.finish(event, None);

    let params = InsertEventParams {
        aggregate_id: &args.aggregate_id.clone().unwrap(),
        id: &args.id.clone().unwrap(),
        payload: &input_fbb.finished_data().to_vec(),
        aggregate_version: args.aggregate_version.clone().unwrap(),
    };

    let mutex = std::sync::Mutex::new(Server::new(ServerConfig {
        data_path: Some(temp_dir.path()),
    }));

    let arc = std::sync::Arc::new(mutex);
    let arc_cloned = std::sync::Arc::clone(&arc);

    std::thread::spawn(move || {
        let mut server_endpoint = String::from("tcp://*:");
        server_endpoint.push_str(&tcp_port.to_string());

        let server_context = arc_cloned.lock().unwrap();

        server_context.serve(server_endpoint, stop_rx).unwrap();
    });

    let mut client_endpoint = String::from("tcp://localhost:");
    client_endpoint.push_str(&tcp_port.to_string());

    let client = Client::connect(client_endpoint).await.unwrap();

    let mut fbb = FlatBufferBuilder::new();

    let insert_event_body = generate_fake_event(&mut fbb, &args);

    println!("send_event: {:?}", insert_event_body.to_be_bytes());

    let insert_event_request_body_args = InsertEventRequestBodyArgs {
        event: Some(insert_event_body),
    };

    let body = InsertEventRequestBody::create(&mut fbb, &insert_event_request_body_args);

    let request_args = RequestArgs {
        body: Some(body.as_union_value()),
        body_type: RequestBody::InsertEvent,
    };

    let request = Request::create(&mut fbb, &request_args);

    fbb.finish(request, None);

    let response_data = client.send(fbb.finished_data()).await.unwrap();

    let response = root_as_response(&response_data).unwrap();

    assert_eq!(
        response.status(),
        ResponseStatus::InvalidAggregateVersionError,
        "should return status InvalidAggregateVersionError"
    );

    stop_tx.send(()).unwrap();
}
