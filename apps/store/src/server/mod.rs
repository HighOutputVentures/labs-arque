use std::error::Error;
use std::sync::mpsc::Receiver;
use std::path::Path;
use arque_common::request_generated::root_as_request;

use crate::store::{Store,RocksDBStore};

struct ControllerContext {
  store: Box<dyn Store>,
}

pub struct ServerConfig<'a> {
  pub data_path: Option<&'a Path>
}
pub struct Server<'a> {
  config: ServerConfig<'a>,
  context: ControllerContext
}

impl<'a> Server<'a> {
  pub fn new(config: ServerConfig<'a>) -> Server<'a> {
    let data_path = config.data_path.unwrap_or(Path::new("./data"));

    let store = RocksDBStore::open(data_path).expect("error occured while openning the database");

    Server {
      config,
      context: ControllerContext {
        store: Box::new(store)
      }
    }
  }

  fn handle_request(&self, req: &[u8]) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let request = root_as_request(req)?;

    println!("request: {:?}", request.body_type());

    Ok(vec![])
  }

  pub fn serve(&self, endpoint: String, shutdown: Receiver<()>) -> Result<(), Box<dyn Error + Send + Sync>> {

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
