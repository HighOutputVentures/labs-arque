mod client;
use std::error::Error;

use arque_common::{
    request_generated::{
        Event, EventBuilder, InsertEventRequestBody, InsertEventRequestBodyArgs, Request,
        RequestArgs, RequestBody,
    },
    response_generated::{root_as_response, ResponseStatus},
};
pub use client::Client;
use flatbuffers::FlatBufferBuilder;

pub struct Driver {
    client: Option<Client>,
    endpoint: String,
}

impl Driver {
    pub fn new(endpoint: String) -> Self {
        Self {
            client: None,
            endpoint,
        }
    }

    async fn get_client(&mut self) -> &Client {
        let client = self.client.clone();
        let endpoint = self.endpoint.clone();

        let ctx = match client {
            Some(context) => context,
            None => Client::connect(endpoint).await.unwrap(),
        };

        self.client = Some(ctx);

        self.client.as_ref().unwrap()
    }

    pub async fn connect(&self) {}

    pub async fn close(&self) {}

    pub async fn insert_event<'a>(
        &mut self,
        event: Event<'a>,
    ) -> Result<ResponseStatus, Box<dyn Error>> {
        let client = self.get_client().await;

        let mut fbb = FlatBufferBuilder::from_vec(event._tab.buf.to_vec());

        let event_builder = EventBuilder::new(&mut fbb);

        let insert_event_request_body_args = InsertEventRequestBodyArgs {
            event: Some(event_builder.finish()),
        };

        let request_args = RequestArgs {
            body: Some(
                InsertEventRequestBody::create(&mut fbb, &insert_event_request_body_args)
                    .as_union_value(),
            ),
            body_type: RequestBody::InsertEvent,
        };

        let request = Request::create(&mut fbb, &request_args);

        fbb.finish(request, None);

        let response_data = client.send(fbb.finished_data()).await.unwrap();

        match root_as_response(&response_data) {
            Ok(response) => Ok(response.status()),

            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn list_aggregate_events<'a>(_event: Event<'a>) {}
}

#[cfg(test)]
mod tests {
    use std::{
        iter::repeat_with,
        sync::mpsc::{channel, Receiver},
        thread,
    };

    use super::*;
    use arque_common::{
        request_generated::EventArgs,
        response_generated::{
            InsertEventResponseBody, InsertEventResponseBodyArgs, Response, ResponseArgs,
            ResponseBody,
        },
    };

    use get_port::{tcp::TcpPort, Ops};
    use rstest::*;

    pub fn random_bytes(len: usize) -> Vec<u8> {
        repeat_with(|| fastrand::u8(..)).take(len).collect()
    }

    pub fn server(
        endpoint: String,
        shutdown: Receiver<()>,
        response: Vec<u8>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let ctx = zmq::Context::new();

        let socket = ctx.socket(zmq::ROUTER)?;

        socket.bind(endpoint.as_str())?;

        loop {
            if !shutdown.try_recv().is_err() {
                break;
            }

            if socket.poll(zmq::PollEvents::POLLIN, 1000).unwrap() != 0 {
                let message = socket.recv_multipart(0).unwrap();

                socket.send(message[0].as_slice(), zmq::SNDMORE).unwrap();
                socket.send(message[1].as_slice(), zmq::SNDMORE).unwrap();
                socket.send(response.to_owned(), 0).unwrap();
            }
        }

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn test_driver_insert_event() {
        let tcp_port = TcpPort::any("127.0.0.1").unwrap();
        let (stop_tx, stop_rx) = channel::<()>();

        let response = {
            let mut fbb = FlatBufferBuilder::new();

            let insert_event_response_body =
                InsertEventResponseBody::create(&mut fbb, &InsertEventResponseBodyArgs {});

            let response = Response::create(
                &mut fbb,
                &ResponseArgs {
                    body_type: ResponseBody::InsertEvent,
                    body: Some(insert_event_response_body.as_union_value()),
                    status: ResponseStatus::Ok,
                },
            );
            fbb.finish(response, None);
            fbb.finished_data().to_owned()
        };

        thread::spawn(move || {
            let mut server_endpoint = String::from("tcp://*:");
            server_endpoint.push_str(&tcp_port.to_string());
            server(server_endpoint, stop_rx, response).unwrap();
        });

        let mut client_endpoint = String::from("tcp://localhost:");
        client_endpoint.push_str(&tcp_port.to_string());

        let mut driver = Driver::new(client_endpoint);

        let mut fbb = FlatBufferBuilder::new();

        let event_args = EventArgs {
            id: Some(fbb.create_vector(&random_bytes(12))),
            type_: fastrand::u16(..),
            aggregate_id: Some(fbb.create_vector(&random_bytes(12))),
            aggregate_version: fastrand::u32(..),
            body: Some(fbb.create_vector(&random_bytes(1024))),
            meta: Some(fbb.create_vector(&random_bytes(64))),
        };

        let event = Event::create(&mut fbb, &event_args);

        fbb.finish(event, None);

        let event_data = fbb.finished_data();

        let event = flatbuffers::root::<Event>(event_data).unwrap();

        let container = driver.insert_event(event).await.unwrap();

        assert_eq!(
            container,
            ResponseStatus::Ok,
            "should return response status ok"
        );

        stop_tx.send(()).unwrap();
    }
}
