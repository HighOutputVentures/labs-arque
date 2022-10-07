use std::{collections::HashMap, sync::Mutex};
use rand::{rngs::ThreadRng, Rng};
use zmq::Socket;

pub struct Client {
  socket: Socket,
  requests: Mutex<HashMap<u16, (String, oneshot::Sender<String>)>>,
  rng: Mutex<ThreadRng>,
}

impl Client {
  fn create_request(&self, msg: String) -> (u16, oneshot::Receiver<String>) {
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

    let (tx, rx) = oneshot::channel::<String>();

    requests.insert(id, (msg, tx));

    (id, rx)
  }

  pub fn connect(endpoint: &str) -> Result<Client, Box<dyn std::error::Error>> {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::REQ).unwrap();

    socket
      .connect(endpoint).unwrap();

    let (tx, rx) = std::sync::mpsc::channel::<String>();

    std::thread::spawn(move || {
      rx.recv();      
    });

    tx.send("".to_string());

    Ok(Client {
      socket,
      requests: Mutex::new(HashMap::new()),
      rng: Mutex::new(rand::thread_rng())
    })
  }

  pub async fn send(&self, msg: String) {
    let (id, rx) = self.create_request(msg);

    rx.await.unwrap();
    // self.socket.send(msg, 0).expect("cannot set message");
    // let _ = self.socket.recv_string(0).expect("unable to receive the response");
  }
}
