use std::{sync::{Arc, Mutex}, collections::HashMap};
use zmq::Socket;

pub struct Client {
  requests: Arc<Mutex<HashMap<u32, oneshot::Sender<Vec<u8>>>>>,
  socket: Arc<Mutex<Socket>>,
}

impl Client {
  async fn establish_connection(&self) -> Result<(), Box<dyn std::error::Error>> {
    // let socket = self.socket.lock().unwrap();

    // send pings

    Ok(())
  }

  async fn start_worker_thread(&self) -> Result<(), Box<dyn std::error::Error>> {
    let requests = self.requests.clone();
    let socket = self.socket.clone();

    std::thread::spawn(move || {
      loop {
        let socket = socket.lock().unwrap();

        if socket.poll(zmq::PollEvents::POLLIN, 0).unwrap() != 0 {
          let mut requests = requests.lock().unwrap();

          let message = socket.recv_multipart(0).unwrap();
          println!("response: {:?}", message);

          if message.len() == 2 {
            let id: &[u8; 4] = message[0].as_slice().try_into().expect("unable to parse the request id");
            let id = u32::from_be_bytes(id.to_owned());

            if let Some(tx) = requests.remove(&id) {
              tx.send(message[1].clone()).expect("unable to receive response");
            }
          }

          drop(requests);
        }

        drop(socket);
      }
    });

    Ok(())
  }

  pub async fn connect(endpoint: String) -> Result<Client, Box<dyn std::error::Error>> {
    let ctx = zmq::Context::new();

    let socket = ctx.socket(zmq::DEALER).expect("unable to create socket");
    socket.connect(endpoint.as_str()).unwrap();

    let socket = Arc::new(Mutex::new(socket));

    let requests: Arc<Mutex<HashMap<u32, oneshot::Sender<Vec<u8>>>>> = Arc::new(Mutex::new(HashMap::new()));

    let client = Client {
      requests,
      socket,
    };

    Client::start_worker_thread(&client).await?;
    Client::establish_connection(&client).await?;

    Ok(client)
  }

  pub async fn send(&self, data: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let id = fastrand::u32(..);

    let (tx, rx) = oneshot::channel::<Vec<u8>>();

    let socket = self.socket.lock().unwrap();
    let mut requests = self.requests.lock().unwrap();

    requests.insert(id, tx);

    drop(requests);

    id.to_be_bytes().to_vec();

    socket.send(id.to_be_bytes().as_slice(), zmq::SNDMORE).unwrap();
    println!("send: {:?}", id.to_be_bytes());
    socket.send(data, 0).unwrap();

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
          let messages = socket.recv_multipart(0).unwrap();
          println!("request: {:?}", messages);

          thread::sleep(Duration::from_millis(fastrand::u64(500..=1000)));

          //reply
          socket.send_multipart(messages, 0).unwrap();
        }
      });

      let client = Client::connect("tcp://localhost:5556".to_string()).await.unwrap();

      thread::sleep(Duration::from_millis(1000));
      client.send(format!("message 1").as_bytes()).await.unwrap();
      thread::sleep(Duration::from_millis(1000));
      client.send(format!("message 2").as_bytes()).await.unwrap();
      thread::sleep(Duration::from_millis(1000));
      join_all([
        client.send(format!("message 3").as_bytes()),
        client.send(format!("message 4").as_bytes()),
      ]).await;
      thread::sleep(Duration::from_millis(1000));
    }
}