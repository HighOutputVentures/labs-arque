use std::{sync::{Arc, Mutex}, collections::HashMap};

use zmq::Socket;

pub struct Client {
  requests: Arc<Mutex<HashMap<u32, oneshot::Sender<String>>>>,
  socket: Arc<Mutex<Socket>>,
}

impl Client {
  pub async fn connect(endpoint: String) -> Result<Client, Box<dyn std::error::Error>> {
    let ctx = zmq::Context::new();

    let socket_ = Arc::new(Mutex::new(ctx.socket(zmq::DEALER).unwrap()));

    let requests_: Arc<Mutex<HashMap<u32, oneshot::Sender<String>>>> = Arc::new(Mutex::new(HashMap::new()));

    let requests = requests_.clone();
    let socket = socket_.clone();
    let (conn_tx, conn_rx) = oneshot::channel::<()>();
    std::thread::spawn(move || {
      // prepare connection
      let socket = socket.lock().unwrap();
      socket.connect(endpoint.as_str()).unwrap();
      drop(socket);

      conn_tx.send(()).unwrap();

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
      requests: requests_,
      socket: socket_,
    })
  }

  pub async fn send(&self, msg: String) -> Result<String, Box<dyn std::error::Error>> {
    let id = fastrand::u32(..);

    let (tx, rx) = oneshot::channel::<String>();

    let mut requests = self.requests.lock().unwrap();

    requests.insert(id, tx);

    drop(requests);

    // send request through socket;

    let socket = self.socket.lock().unwrap();

    socket.send(id.to_be_bytes().as_slice(), zmq::SNDMORE).unwrap();
    println!("send: {:?}", id.to_be_bytes());
    socket.send(&msg, 0).unwrap();

    drop(socket);

    let response = rx.await.unwrap();

    Ok(response)
  }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread,time::Duration};
    use futures::future::join_all;
    use rstest::*;

    #[rstest]
    #[tokio::test]
    async fn send_test() {
      thread::spawn(|| {
        let ctx = zmq::Context::new();
    
        let socket = ctx.socket(zmq::ROUTER).unwrap();
    
        socket.bind("tcp://*:5556").expect("failed to bind socket");

        loop {
          let mut envelope = zmq::Message::new();
          let mut identity = zmq::Message::new();
          let mut message = zmq::Message::new();
          socket.recv(&mut envelope, 0).unwrap();
          socket.recv(&mut identity, 0).unwrap();
          println!("identity: {:?}", identity);
          socket.recv(&mut message, 0).unwrap();
          println!("message: {:?}", message);

          thread::sleep(Duration::from_millis(500));

          //reply
          socket.send(envelope, zmq::SNDMORE).unwrap();
          socket.send(identity, zmq::SNDMORE).unwrap();
          socket.send(message, 0).unwrap();
        }
      });

      let client = Client::connect("tcp://localhost:5556".to_string()).await.unwrap();

      join_all([
        client.send(format!("message 1")),
        client.send(format!("message 2")),
        client.send(format!("message 3")),
      ]).await;

      thread::sleep(Duration::from_millis(2000));
    }
}