mod store;
mod client;
mod controller;

use std::thread;
use std::time::Duration;

#[allow(dead_code, unused_variables)]

// async fn send_request(mut rec: Receiver<&'static str>) {}

async fn client_task() {
    tokio::spawn(async move {
        let ctx = zmq::Context::new();

        let client = ctx.socket(zmq::DEALER).unwrap();
        client.set_identity("client1".as_bytes()).unwrap();
        client
            .connect("tcp://localhost:5555")
            .expect("failed to connect client");

        client.send("Hello", 0).unwrap();
        println!("client message sent");

        loop {
            let reply = client.recv_string(0).unwrap().unwrap();
            println!("response from worker: {}", reply);
        }
    })
    .await
    .unwrap();

    // let _ = tokio::join!(
    //     send_request(rx),
    //     send_request(rx2),
    //     send_request(rx3),
    //     send_request(rx4)
    // );
}

async fn server_task() {
    let ctx = zmq::Context::new();

    let router = ctx.socket(zmq::ROUTER).unwrap();
    let dealer = ctx.socket(zmq::DEALER).unwrap();

    router.bind("tcp://*:5555").expect("failed to bind router");
    dealer
        .bind("inproc://workers")
        .expect("failed to bind dealer");

    for _ in 0..3 {
        let ctx = ctx.clone();
        thread::spawn(move || worker_task(&ctx));
    }

    zmq::proxy(&router, &dealer).expect("failed proxying");
}

fn worker_task(context: &zmq::Context) {
    let worker = context.socket(zmq::DEALER).unwrap();
    worker
        .connect("inproc://workers")
        .expect("failed to connect worker");

    println!("worker started");

    loop {
        let msg = worker.recv_string(0).unwrap().unwrap();

        println!("received from client: {}", msg);
        // let data = worker.recv(&mut msg, 0).unwrap();
        // let id = worker
        //     .recv_string(0)
        //     .expect("worker failed receiving")
        //     .unwrap();
        // let data = worker
        //     .recv_string(0)
        //     .expect("worker failed receiving")
        //     .unwrap();
        // println!("received from client: {:?}", data);
        thread::sleep(Duration::from_millis(1000));
        worker.send("client1", zmq::SNDMORE).unwrap();
        worker.send("World", 0).unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let _ = thread::spawn(|| {
    server_task()
  }).join().unwrap();

  Ok(())
}
