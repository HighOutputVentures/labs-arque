mod helpers;

use arque_common::{
    request_generated::{
        ListAggregateEventsRequestBody, ListAggregateEventsRequestBodyArgs, Request, RequestArgs,
        RequestBody,
    },
    response_generated::{root_as_response, ResponseStatus},
};
use arque_driver::Client;
use arque_store::{InsertEventParams, RocksDBStore, Server, ServerConfig, Store};
use flatbuffers::FlatBufferBuilder;
use get_port::{tcp::TcpPort, Ops};
use helpers::{generate_fake_event, random_bytes, GenerateFakeEventArgs};
use rstest::*;
use std::sync::mpsc::channel;
use tempdir::TempDir;

#[rstest]
#[tokio::test]
async fn test_list_aggregate_events() {
    let (stop_tx, stop_rx) = channel::<()>();

    let tcp_port = TcpPort::any("127.0.0.1").unwrap();

    let temp_dir = TempDir::new("arque_test").unwrap();

    let store = RocksDBStore::open(temp_dir.path()).unwrap();

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

    input_fbb.finish(event, None);

    let params = InsertEventParams {
        aggregate_id: &args.aggregate_id.clone().unwrap(),
        id: &args.id.clone().unwrap(),
        payload: &input_fbb.finished_data().to_vec(),
        aggregate_version: args.aggregate_version.clone().unwrap(),
    };

    store.insert_event(params).unwrap();

    drop(store);

    std::thread::spawn(move || {
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

    let list_aggregate_events_request_body_args = ListAggregateEventsRequestBodyArgs {
        aggregate_id: Some(fbb.create_vector(&args.aggregate_id.unwrap())),
        aggregate_version: args.aggregate_version.unwrap(),
        limit: 1,
    };

    let body =
        ListAggregateEventsRequestBody::create(&mut fbb, &list_aggregate_events_request_body_args);

    let request_args = RequestArgs {
        body: Some(body.as_union_value()),
        body_type: RequestBody::ListAggregateEvents,
    };

    let request = Request::create(&mut fbb, &request_args);

    fbb.finish(request, None);

    let response_data = client.send(fbb.finished_data()).await.unwrap();

    let response = root_as_response(&response_data).unwrap();

    let events = response
        .body_as_list_aggregate_events()
        .unwrap()
        .events()
        .unwrap();

    assert_eq!(
        response.status(),
        ResponseStatus::Ok,
        "should return status Ok"
    );

    assert_eq!(events.len(), 1, "should return len equal to 1");

    stop_tx.send(()).unwrap();
}


#[rstest]
#[tokio::test]
async fn test_empty_list() {
    let (stop_tx, stop_rx) = channel::<()>();

    let tcp_port = TcpPort::any("127.0.0.1").unwrap();

    let temp_dir = TempDir::new("arque_test").unwrap();



    std::thread::spawn(move || {
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

    let list_aggregate_events_request_body_args = ListAggregateEventsRequestBodyArgs {
        aggregate_id: Some(fbb.create_vector(&random_bytes(12))),
        aggregate_version: fastrand::u32(..),
        limit: 1,
    };

    let body =
        ListAggregateEventsRequestBody::create(&mut fbb, &list_aggregate_events_request_body_args);

    let request_args = RequestArgs {
        body: Some(body.as_union_value()),
        body_type: RequestBody::ListAggregateEvents,
    };

    let request = Request::create(&mut fbb, &request_args);

    fbb.finish(request, None);

    let response_data = client.send(fbb.finished_data()).await.unwrap();

    let response = root_as_response(&response_data).unwrap();

    let events = response
        .body_as_list_aggregate_events()
        .unwrap()
        .events()
        .unwrap();

    assert_eq!(
        response.status(),
        ResponseStatus::Ok,
        "should return status Ok"
    );

    assert_eq!(events.len(), 0, "should return len equal to 0");

    stop_tx.send(()).unwrap();
}
