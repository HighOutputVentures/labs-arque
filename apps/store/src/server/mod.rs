use std::thread;

use futures::{channel::oneshot};

fn serve(endpoint: String, handler: fn(&[u8]) -> Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
  let ctx = zmq::Context::new();

  let socket = ctx.socket(zmq::ROUTER)?;

  socket.bind(endpoint.as_str())?;
    
  loop {
    if socket.poll(zmq::PollEvents::POLLIN, 0).expect("error occured while polling the scoket") != 0 {
      let message = socket.recv_multipart(0).unwrap();
      println!("request: {:?}", message);

      let response = handler(message[2].as_slice());

      socket.send(message[0].as_slice(), zmq::SNDMORE).unwrap();
      socket.send(message[1].as_slice(), zmq::SNDMORE).unwrap();
      socket.send(response, 0).unwrap();
    } else {
        break;
    }
  }

  Ok(())
}

pub struct Server {}

impl Server {
  pub fn bind(endpoint: String, handler: fn(&[u8]) -> Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let future = async {
      let endpoint = endpoint.clone();
      let handler = handler.clone();

      let (tx, rx) = oneshot::channel::<()>();

      thread::spawn(move || {
        let result = serve(endpoint, handler);

        tx.send(()).unwrap();
      });

      rx.await;

      ()
    };

    Ok(())
  }
}
