use crate::{
    controllers::{insert_event, ControllerContext, InsertEventError},
    stream::KafkaStream,
};
use arque_common::{request_generated::{root_as_request, RequestBody}, response_generated::{Response, ResponseArgs, ResponseBody, ResponseStatus}};
use flatbuffers::FlatBufferBuilder;
use std::error::Error;
use std::path::Path;
use std::sync::mpsc::Receiver;

use crate::store::RocksDBStore;

#[allow(dead_code)]
pub struct ServerConfig<'a> {
    pub data_path: Option<&'a Path>,
}

#[allow(dead_code)]
pub struct Server<'a> {
    config: ServerConfig<'a>,
    context: ControllerContext,
}

#[allow(dead_code)]
impl<'a> Server<'a> {
    pub fn new(config: ServerConfig<'a>) -> Server<'a> {
        let data_path = config.data_path.unwrap_or(Path::new("./data"));

        let store =
            RocksDBStore::open(data_path).expect("error occured while openning the database");

        let stream = KafkaStream {
            broker: "localhost:9092".to_string(),
        };

        Server {
            config,
            context: ControllerContext {
                store: Box::new(store),
                stream: Box::new(stream),
            },
        }
    }

    fn handle_request(&self, req: &[u8]) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let request = root_as_request(req)?;

        let mut fbb = FlatBufferBuilder::new();

        let response = match request.body_type() {
            RequestBody::InsertEvent => {
                match insert_event(&self.context, &request.body_as_insert_event().unwrap()) {
                    Err(e) => {
                        match e {
                            InsertEventError::InvalidAggregateVersion => Response::create(&mut fbb, &ResponseArgs {
                                body_type: ResponseBody::InsertEvent,
                                body: None,
                                status: ResponseStatus::InvalidAggregateVersionError,
                            }),
                            _ => Response::create(&mut fbb, &ResponseArgs {
                                body_type: ResponseBody::InsertEvent,
                                body: None,
                                status: ResponseStatus::UnknownError,
                            })
                        }
                    },
                    Ok(()) => Response::create(&mut fbb, &ResponseArgs {
                        body_type: ResponseBody::InsertEvent,
                        body: None,
                        status: ResponseStatus::Ok,
                    }),
                }
            },
            _ => {
                Response::create(&mut fbb, &ResponseArgs {
                    body_type: ResponseBody::InsertEvent,
                    body: None,
                    status: ResponseStatus::UnknownError,
                })
            }
        };

        fbb.finish(response, None);

        Ok(fbb.finished_data().to_owned())
    }

    pub fn serve(
        &self,
        endpoint: String,
        shutdown: Receiver<()>,
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

                let response = self.handle_request(message[2].as_slice())?;

                socket.send(message[0].as_slice(), zmq::SNDMORE).unwrap();
                socket.send(message[1].as_slice(), zmq::SNDMORE).unwrap();
                socket.send(response, 0).unwrap();
            }
        }

        Ok(())
    }
}
