use std::thread;
use std::time::Duration;

async fn send_request() {
  println!("request");

  // do something here;

  println!("reply");
}

async fn client_task() {
  // initialize channel

  tokio::spawn(async {
    let ctx = zmq::Context::new();

    let client = ctx.socket(zmq::REQ).unwrap();
    client.connect("tcp://localhost:5555").expect("failed to connect client");
  
    client.send("Hello", 0).unwrap();
    let reply = client.recv_string(0).unwrap().unwrap();
  
    println!("reply: {}", reply);
  });

  let _ = tokio::join!(
    send_request(),
    send_request(),
    send_request(),
    send_request()
  );
}

async fn server_task() {
  let ctx = zmq::Context::new();

  let router = ctx.socket(zmq::ROUTER).unwrap();
  let dealer = ctx.socket(zmq::DEALER).unwrap();

  router.bind("tcp://*:5555").expect("failed to bind router");
  dealer.bind("inproc://workers").expect("failed to bind dealer");

  for _ in 0..3 {
    let ctx = ctx.clone();
    thread::spawn(move || worker_task(&ctx));
  }

  zmq::proxy(&router, &dealer).expect("failed proxying");
}

fn worker_task(context: &zmq::Context) {
  let worker = context.socket(zmq::REP).unwrap();
  worker
      .connect("inproc://workers")
      .expect("failed to connect worker");

  println!("worker started");

  loop {
    worker
      .recv_string(0)
      .expect("worker failed receiving")
      .unwrap();
    thread::sleep(Duration::from_millis(1000));
    worker.send("World", 0).unwrap();
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let _ = tokio::join!(
    tokio::spawn(server_task()),
    tokio::spawn(client_task()),
    tokio::spawn(client_task()),
    tokio::spawn(client_task())
  );
  
  Ok(())
}
