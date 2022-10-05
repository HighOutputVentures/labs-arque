use std::collections::HashMap;
use rand::{rngs::ThreadRng, Rng};
use zmq::Socket;

pub struct Client {
  socket: Socket,
  requests: HashMap<u16, String>,
  rng: ThreadRng,
}

impl Client {
  pub fn connect(endpoint: &str) -> Result<Client, Box<dyn std::error::Error>> {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::REQ).unwrap();

    socket
      .connect(endpoint).unwrap();

    Ok(Client {
      socket,
      requests: HashMap::new(),
      rng: rand::thread_rng()
    })
  }

  pub async fn send(&mut self, msg: &str) {
    let id = {
      let mut id: u16;
      loop {
        id = self.rng.gen();
  
        if !self.requests.contains_key(&id) {
          break;
        }
      }

      id
    };

    self.requests.insert(id, msg.to_string());

    self.socket.send(msg, 0).expect("cannot set message");
    let _ = self.socket.recv_string(0).expect("unable to receive the response");
  }
}
