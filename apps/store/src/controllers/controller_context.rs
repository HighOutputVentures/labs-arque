use crate::store::{Store, self};

pub struct ControllerContext {
  pub store: dyn Store,
}

pub fn handle_request(ctx: &ControllerContext, data: &Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
  /**
   * - deserialize data
   * - determine the request type
   * - call the corresponding controller
   */
  Ok(vec![])
}
