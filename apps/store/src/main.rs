mod store;
mod controllers;
mod server;
mod stream;

use server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  Server::bind("tcp://*:4000".to_string(), |req| {
    println!("{:?}", req);

    vec![]
  })?;

  Ok(())
}
