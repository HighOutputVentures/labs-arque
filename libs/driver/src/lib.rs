mod client;
mod types;

pub use client::Client;

use arque_common::{
    request_generated::{
        EventArgs, InsertEventRequestBody, InsertEventRequestBodyArgs,
        ListAggregateEventsRequestBody, ListAggregateEventsRequestBodyArgs, Request, RequestArgs,
        RequestBody,
    },
    response_generated::{root_as_response, ResponseStatus},
};

use custom_error::custom_error;
use flatbuffers::FlatBufferBuilder;
use types::ListAggregateEventsParams;

custom_error! {
    pub InsertEventError
    Unknown{message:String} = "unknown: {message}"
}

custom_error! {
    pub ListAggregateEventsError
    Unknown{message:String} = "unknown: {message}"
}

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
        event: arque_common::event_generated::Event<'a>,
    ) -> Result<ResponseStatus, InsertEventError> {
        let client = self.get_client().await;

        let mut fbb = FlatBufferBuilder::new();

        let event_args = EventArgs {
            id: Some(fbb.create_vector(event.id().unwrap())),
            type_: event.type_(),
            aggregate_id: Some(fbb.create_vector(event.aggregate_id().unwrap())),
            aggregate_version: event.aggregate_version(),
            body: Some(fbb.create_vector(event.body().unwrap())),
            meta: Some(fbb.create_vector(event.meta().unwrap())),
        };

        let event_object = arque_common::request_generated::Event::create(&mut fbb, &event_args);

        let insert_event_request_body_args = InsertEventRequestBodyArgs {
            event: Some(event_object),
        };

        let insert_event_request_body =
            InsertEventRequestBody::create(&mut fbb, &insert_event_request_body_args)
                .as_union_value();

        let request_args = RequestArgs {
            body: Some(insert_event_request_body),
            body_type: RequestBody::InsertEvent,
        };

        let request = Request::create(&mut fbb, &request_args);

        fbb.finish(request, None);

        let payload = fbb.finished_data();

        let response_data = match client.send(payload).await {
            Ok(data) => data,
            Err(e) => {
                return Err(InsertEventError::Unknown {
                    message: e.to_string(),
                })
            }
        };

        match root_as_response(&response_data) {
            Ok(response) => Ok(response.status()),

            Err(e) => Err(InsertEventError::Unknown {
                message: e.to_string(),
            }),
        }
    }

    pub async fn list_aggregate_events<'a>(
        &mut self,
        list_aggregate_events_params: ListAggregateEventsParams<'a>,
        buffer: &'a mut Vec<u8>,
    ) -> Result<Vec<arque_common::event_generated::Event<'a>>, ListAggregateEventsError> {
        let client = self.get_client().await;

        let mut fbb = FlatBufferBuilder::new();
        let list_aggregate_events_request_body_args = ListAggregateEventsRequestBodyArgs {
            aggregate_id: Some(fbb.create_vector(list_aggregate_events_params.aggregate_id)),
            aggregate_version: list_aggregate_events_params.aggregate_version.unwrap(),
            limit: list_aggregate_events_params.limit,
        };

        let list_aggregate_events_request_body = ListAggregateEventsRequestBody::create(
            &mut fbb,
            &list_aggregate_events_request_body_args,
        )
        .as_union_value();

        let request_args = RequestArgs {
            body: Some(list_aggregate_events_request_body),
            body_type: RequestBody::InsertEvent,
        };

        let request = Request::create(&mut fbb, &request_args);

        fbb.finish(request, None);

        let payload = fbb.finished_data();

        let response_data = match client.send(payload).await {
            Ok(data) => data,
            Err(e) => {
                return Err(ListAggregateEventsError::Unknown {
                    message: e.to_string(),
                })
            }
        };

        for (_, data) in response_data.iter().enumerate() {
            buffer.push(*data);
        }

        let response = match root_as_response(buffer) {
            Ok(data) => data,
            Err(e) => {
                return Err(ListAggregateEventsError::Unknown {
                    message: e.to_string(),
                })
            }
        };

        let list = response.body_as_list_aggregate_events().unwrap();
        let events = list.events().unwrap();

        let mut event_vec: Vec<arque_common::event_generated::Event> = Vec::new();

        for (_, event) in events.iter().enumerate() {
            let event_data = arque_common::event_generated::Event::init_from_table(event._tab.to_owned());

            event_vec.push(event_data);
        }

        Ok(event_vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arque_common::response_generated::{
        InsertEventResponseBody, InsertEventResponseBodyArgs, ListAggregateEventsResponseBody,
        ListAggregateEventsResponseBodyArgs, Response, ResponseArgs, ResponseBody,
    };
    use std::error::Error;
    use std::{
        iter::repeat_with,
        sync::mpsc::{channel, Receiver},
        thread,
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

        let event_args = arque_common::event_generated::EventArgs {
            id: Some(fbb.create_vector(&random_bytes(12))),
            type_: fastrand::u16(..),
            aggregate_id: Some(fbb.create_vector(&random_bytes(12))),
            aggregate_version: 1,
            body: Some(fbb.create_vector(&random_bytes(12))),
            meta: Some(fbb.create_vector(&random_bytes(12))),
        };

        let event = arque_common::event_generated::Event::create(&mut fbb, &event_args);

        fbb.finish(event, None);

        let event_data = fbb.finished_data();

        let event_object = flatbuffers::root::<arque_common::event_generated::Event>(event_data).unwrap();

        let response_status = driver.insert_event(event_object).await.unwrap();

        assert_eq!(
            response_status,
            ResponseStatus::Ok,
            "should return response status ok"
        );

        stop_tx.send(()).unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn test_driver_list_aggregate_events() {
        let tcp_port = TcpPort::any("127.0.0.1").unwrap();
        let (stop_tx, stop_rx) = channel::<()>();

        let aggregate_id = random_bytes(12);
        let aggregate_id_clone = aggregate_id.clone();

        let response = {
            let mut fbb = FlatBufferBuilder::new();

            let mut events: Vec<flatbuffers::WIPOffset<arque_common::response_generated::Event>> =
                Vec::new();

            let event_args = arque_common::response_generated::EventArgs {
                id: Some(fbb.create_vector(&random_bytes(12))),
                type_: fastrand::u16(..),
                aggregate_id: Some(fbb.create_vector(&aggregate_id_clone)),
                aggregate_version: 1,
                body: Some(fbb.create_vector(&random_bytes(12))),
                meta: Some(fbb.create_vector(&random_bytes(12))),
            };

            events.push(arque_common::response_generated::Event::create(
                &mut fbb,
                &event_args,
            ));

            let events_array = fbb.create_vector(&events);

            let list_aggregate_events_response_body = ListAggregateEventsResponseBody::create(
                &mut fbb,
                &ListAggregateEventsResponseBodyArgs {
                    events: Some(events_array),
                },
            );

            let response = Response::create(
                &mut fbb,
                &ResponseArgs {
                    body_type: ResponseBody::ListAggregateEvents,
                    body: Some(list_aggregate_events_response_body.as_union_value()),
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

        let list_aggregate_events_params = ListAggregateEventsParams {
            aggregate_id: &aggregate_id,
            aggregate_version: Some(1u32),
            limit: 1u32,
        };

        let mut buffer: Vec<u8> = Vec::new();

        let events = driver
            .list_aggregate_events(list_aggregate_events_params, &mut buffer)
            .await
            .unwrap();

        assert_eq!(events.len(), 1, "should return events length equal to 1");

        stop_tx.send(()).unwrap();
    }
}
