use std::{collections::HashMap, sync::Mutex};
use rand::{rngs::ThreadRng, Rng};
use zmq::Socket;

pub struct Client {
  socket: Socket,
  requests: Mutex<HashMap<u16, String>>,
  rng: Mutex<ThreadRng>,
}

impl Client {
  fn generate_request_id(&self) -> u8 {
    let mut rng = self.rng.lock().unwrap();
  }

  pub fn connect(endpoint: &str) -> Result<Client, Box<dyn std::error::Error>> {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::REQ).unwrap();

    socket
      .connect(endpoint).unwrap();

    Ok(Client {
      socket,
      requests: Mutex::new(HashMap::new()),
      rng: Mutex::new(rand::thread_rng())
    })
  }

  pub async fn send(&self, msg: String) {
    let mut rng = self.rng.lock().unwrap();
    let mut requests = self.requests.lock().unwrap();
    
    let id = {
      let mut id: u16;
      loop {
        id = rng.gen();
  
        if !requests.contains_key(&id) {
          break;
        }
      }

      id
    };

    println!("{}", id);

    requests.insert(id, msg);

    // self.socket.send(msg, 0).expect("cannot set message");
    // let _ = self.socket.recv_string(0).expect("unable to receive the response");
  }
}
