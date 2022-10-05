mod client;

use std::{thread, time::Duration};
use crate::client::Client;

fn worker_task(context: &zmq::Context, id: u8) {
  let worker = context.socket(zmq::REP).unwrap();
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
}

fn main() {
  thread::spawn(|| {
    let ctx = zmq::Context::new();

    let router = ctx.socket(zmq::ROUTER).unwrap();
    let dealer = ctx.socket(zmq::DEALER).unwrap();

    router.bind("tcp://*:5555").expect("failed to bind router");
    dealer
        .bind("inproc://workers")
        .expect("failed to bind dealer");

    for id in 0..3 {
        let ctx = ctx.clone();
        thread::spawn(move || worker_task(&ctx, id));
    }
    println!("proxy");
    zmq::proxy(&router, &dealer).expect("failed proxying");
  });

  tokio::runtime::Builder::new_multi_thread()
    .build()
    .unwrap()
    .block_on(async {
      let mut client = Client::connect("tcp://localhost:5555").unwrap();

      let mut handles = vec![];

      handles.push(client.send("message 0"));

      futures::future::join_all(handles).await;
    });

  println!("end");
}
