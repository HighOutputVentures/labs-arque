mod controllers;
mod server;
mod store;
mod stream;

use server::{Server, ServerConfig};
use std::error::Error;
use std::sync::mpsc::channel;
use tempdir::TempDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (_tx, _rx) = channel::<()>();

    // ctrlc::set_handler(move || {
    //   tx.send(()).unwrap();
    // })?;

    // Server::serve("tcp://*:4000".to_string(), |req| {
    //   println!("{:?}", req);

    //   vec![]
    // }, rx)?;
 
    let server_endpoint = String::from("tcp://*:4000");
    let server_temp = server_endpoint.clone();
    let temp_dir = TempDir::new("arque_test").unwrap();
   
    let server = Server::new(ServerConfig {
        data_path: Some(temp_dir.path()),
    });

    println!("path: {:?}", temp_dir.path());
    println!("server started @ {:?}", server_temp);

    server.serve(server_endpoint, _rx).unwrap();

  

    Ok(())
}
