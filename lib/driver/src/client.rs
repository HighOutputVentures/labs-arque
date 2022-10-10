use std::{collections::HashMap, sync::Mutex};
use rand::{rngs::ThreadRng, Rng};
use std::sync::mpsc::Sender;

pub struct Client {
  requests: Mutex<HashMap<u16, oneshot::Sender<String>>>,
  rng: Mutex<ThreadRng>,
  sender: Sender<(u16, String)>,
}

impl Client {
  fn send_request(&self, msg: String) -> (u16, oneshot::Receiver<String>) {
    let mut rng = self.rng.lock().unwrap();
    let mut requests = self.requests.lock().unwrap();
    let sender = self.sender.clone();
    
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

    requests.insert(id, tx);
    sender.send((id, msg));

    (id, rx)
  }

  pub fn connect(endpoint: &str) -> Result<Client, Box<dyn std::error::Error>> {
    let ctx = zmq::Context::new();
    let socket = ctx.socket(zmq::REQ).unwrap();

    socket
      .connect(endpoint).unwrap();

    let (tx, rx) = std::sync::mpsc::channel::<(u16, String)>();

    std::thread::spawn(move || {
      loop {
        let (id, msg) = rx.recv().unwrap();
        socket.send(&msg, 0);

        println!("id: {}", id);
      }
    });

    Ok(Client {
      requests: Mutex::new(HashMap::new()),
      rng: Mutex::new(rand::thread_rng()),
      sender: tx,
    })
  }

  pub async fn send(&self, msg: String) {
    let (id, rx) = self.send_request(msg);

    rx.await.unwrap();
    // self.socket.send(msg, 0).expect("cannot set message");
    // let _ = self.socket.recv_string(0).expect("unable to receive the response");
  }
}
