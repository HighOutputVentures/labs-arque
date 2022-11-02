use std::{sync::{mpsc::Sender, Arc, Mutex}, collections::HashMap};

pub struct Client {
  sender: Sender<(u32, String, oneshot::Sender<String>)>,
}

impl Client {
  fn send_request(&self, msg: String) -> Result<oneshot::Receiver<String>, Box<dyn std::error::Error>> {
    let id = fastrand::u32(..);

    let sender = self.sender.clone();
    
    let (tx, rx) = oneshot::channel::<String>();

    sender.send((id, msg, tx))?;

    Ok(rx)
  }

  pub async fn connect(endpoint: String) -> Result<Client, Box<dyn std::error::Error>> {
    let ctx = zmq::Context::new();

    let socket = ctx.socket(zmq::DEALER).unwrap();

    let (tx, rx) = std::sync::mpsc::channel::<(u32, String, oneshot::Sender<String>)>();

    let requests_: Arc<Mutex<HashMap<u32, oneshot::Sender<String>>>> = Arc::new(Mutex::new(HashMap::new()));

    let (conn_tx, conn_rx) = oneshot::channel::<()>();
    let requests = requests_.clone();
    std::thread::spawn(move || {
      // prepare connection
      socket.connect(endpoint.as_str()).unwrap();

      conn_tx.send(()).unwrap();

      loop {
        let (id, msg, tx) = rx.recv().unwrap();

        let mut requests = requests.lock().unwrap();

        requests.insert(id, tx);

        drop(requests);

        println!("id: {}, message: \"{}\"", id, msg);
      }
    });

    let requests = requests_.clone();
    std::thread::spawn(move || {
      loop {
        let mut requests = requests.lock().unwrap();

        let keys: Vec<u32> = requests.keys().map(|v| *v).collect();

        for id in keys {
          let tx = requests.remove(&id).unwrap();

          tx.send(format!("response: {}", id)).unwrap();
        }

        drop(requests);
      }
    });

    conn_rx.await?;

    Ok(Client {
      sender: tx,
    })
  }

  pub async fn send(&self, msg: String) -> Result<String, Box<dyn std::error::Error>> {
    let rx = self.send_request(msg).unwrap();
    let response = rx.await.unwrap();

    println!("response: \"{}\"", response);

    Ok(response)
  }
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};
    use super::*;
    use futures::future::join_all;
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

        zmq::proxy(&router, &dealer).expect("failed proxying");
      });

      let client = Client::connect("tcp://localhost:5555".to_string()).await.unwrap();

      join_all([
        client.send(format!("message 1")),
        client.send(format!("message 2")),
      ]).await;
    }
}