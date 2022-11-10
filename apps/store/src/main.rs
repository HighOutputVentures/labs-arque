mod store;
mod controllers;
mod server;
use std::error::Error;
use std::sync::mpsc::channel;
use server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
  let (tx, rx) = channel::<()>();

  ctrlc::set_handler(move || {
    tx.send(()).unwrap();
  })?;

  Server::serve("tcp://*:4000".to_string(), |req| {
    println!("{:?}", req);

    vec![]
  }, rx)?;

  Ok(())
}
