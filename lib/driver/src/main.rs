fn main() {
  let ctx = zmq::Context::new();

  let client = ctx.socket(zmq::REQ).unwrap();
  assert!(client.connect("tcp://localhost:5555").is_ok());
}
