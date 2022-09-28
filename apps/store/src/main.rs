mod store;
mod client;

use std::thread;
use std::time::Duration;
use client::Client;
use tokio::sync::watch::{self, Receiver};

async fn send_request(mut rec: Receiver<&'static str>) {
  // TODO async request
  let task = tokio::spawn(async move {
    let ctx = zmq::Context::new();

    let client = ctx.socket(zmq::REQ).unwrap();

    client
      .connect("tcp://localhost:5555")
      .expect("failed to connect client");

    while rec.changed().await.is_ok() {
      let value = *rec.borrow();
      println!("receive from client task(channel): {}", value);
      client.send(value, 0).unwrap();
    }

    let reply = client.recv_string(0).unwrap().unwrap();

    println!("response from worker: {}", reply);
  });

  task.await.unwrap();
}

async fn client_task() {
  let (tx, rx) = watch::channel("sample");

  let rx2 = rx.clone();
  let rx3 = rx.clone();
  let rx4 = rx.clone();

  // run the client thread
  // open a DEALER socket
  // start an infinite loop
  // - receive requests from tx
  // - send requests through the DEALER

  let client = tokio::spawn(async move {
    tx.send("Hello").unwrap();
  });

  client.await.unwrap();

  let _ = tokio::join!(
    send_request(rx),
    send_request(rx2),
    send_request(rx3),
    send_request(rx4)
  );
}

async fn client_task_new() {
  Client::connect("tcp://localhost:5555").await.unwrap();
}

fn server_task() {
  let ctx = zmq::Context::new();

  let worker = ctx.socket(zmq::REP).unwrap();
  worker
    .bind("tcp://*:5555")
    .expect("failed to connect worker");

  loop {
    let data = worker
      .recv_string(0)
      .expect("worker failed receiving")
      .unwrap();
    println!("received from client: {}", data);
    thread::sleep(Duration::from_millis(1000));
    worker.send("World", 0).unwrap();
  }

  // let router = ctx.socket(zmq::ROUTER).unwrap();
  // let dealer = ctx.socket(zmq::DEALER).unwrap();

  // router.bind("tcp://*:5555").expect("failed to bind router");
  // dealer
  //   .bind("inproc://workers")
  //   .expect("failed to bind dealer");

  // for _ in 0..3 {
  //   let ctx = ctx.clone();
  //   thread::spawn(move || worker_task(&ctx));
  // }

  // zmq::proxy(&router, &dealer).expect("failed proxying");
}

fn worker_task(context: &zmq::Context) {
  let worker = context.socket(zmq::REP).unwrap();
  worker
    .connect("inproc://workers")
    .expect("failed to connect worker");

  println!("worker started");

  loop {
    let data = worker
      .recv_string(0)
      .expect("worker failed receiving")
      .unwrap();
    println!("received from client: {}", data);
    thread::sleep(Duration::from_millis(1000));
    worker.send("World", 0).unwrap();
  }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  thread::spawn(|| {
    client_task_new()
  });

  thread::spawn(|| {
    server_task()
  }).join().unwrap();

  Ok(())
}
