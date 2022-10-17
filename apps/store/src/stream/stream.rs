pub trait Stream {
  fn send(&self, id: String, data: Vec<u8>) -> Result<()>;
}
