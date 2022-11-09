pub struct Server {}

impl Server {
  pub fn bind(endpoint: String, handler: fn(&[u8]) -> Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = zmq::Context::new();

    let socket = ctx.socket(zmq::ROUTER).expect("unable to create socket");
    socket.bind(endpoint.as_str()).expect("unable to bind socket");

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
}