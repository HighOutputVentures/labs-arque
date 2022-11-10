use std::error::Error;
use std::sync::mpsc::Receiver;

pub struct Server {}

impl Server {
  pub fn serve(endpoint: String, handler: fn(&[u8]) -> Vec<u8>, shutdown: Receiver<()>) -> Result<(), Box<dyn Error + Send + Sync>> {
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
      }

      if !shutdown.try_recv().is_err() {
        break;
      }
    }
  
    Ok(())
  }
}
