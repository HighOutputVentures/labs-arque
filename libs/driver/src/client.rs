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
    println!("here");
    rx.await.unwrap();
    // self.socket.send(msg, 0).expect("cannot set message");
    // let _ = self.socket.recv_string(0).expect("unable to receive the response");
  }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};
    use super::*;
    use rstest::*;

    #[rstest]
    #[tokio::test]
    async fn send_test() {
      thread::spawn(|| {
        let ctx = zmq::Context::new();
    
        let router = ctx.socket(zmq::ROUTER).unwrap();
        let dealer = ctx.socket(zmq::DEALER).unwrap();
    
        router.bind("tcp://*:5555").expect("failed to bind router");
        dealer
            .bind("inproc://workers")
            .expect("failed to bind dealer");
    
        for id in 0..3 {
            thread::spawn(move || {
              let ctx = zmq::Context::new();

              let worker = ctx.socket(zmq::REP).unwrap();
              worker
                  .connect("inproc://workers")
                  .expect("failed to connect worker");
            
              println!("worker started");
            
              loop {
                  let msg = worker.recv_string(0).unwrap().unwrap();
                  println!("worker {}: {}", id, msg);
                  thread::sleep(Duration::from_millis(1000));
                  worker.send("pong", 0).unwrap();
              }
            });
        }

        println!("proxy");
        zmq::proxy(&router, &dealer).expect("failed proxying");
      });

      let client = Client::connect("tcp://localhost:5555").unwrap();

      client.send(format!("message: 1")).await;
      client.send(format!("message: 2")).await;
    }
}