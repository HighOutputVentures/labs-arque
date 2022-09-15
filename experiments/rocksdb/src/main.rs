use rocksdb::{DB, Options};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let db = DB::open_default("data.db").unwrap();
 
  Ok(())
}
