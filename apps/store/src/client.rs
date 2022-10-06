use std::{thread, sync::mpsc::channel};

#[allow(dead_code, unused_variables)]

pub struct Client {}

impl Client {
  pub async fn _connect(endpoint: &str) -> Result<Client, Box<dyn std::error::Error>> {
    let endpoint = endpoint.to_owned();

    let (_tx, rx) = channel::<i32>();

    thread::spawn(move || {
      let ctx = zmq::Context::new();
      let client = ctx.socket(zmq::REQ).unwrap();
  
      client
        .connect(endpoint.as_str()).unwrap();
      println!("here");
      loop {
        for x in rx.try_iter() {
          println!("Got: {x}");
        }
        
        client.send("hello", 0).unwrap();
        println!("send");
        thread::sleep(std::time::Duration::from_millis(500));
      }
    });

    Ok(Client {})
  }

  pub async fn _send(&self, _value: i32) {}
}
