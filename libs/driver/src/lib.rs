mod client;

pub use arque_common::request_generated::Event;
pub use client::Client;

pub struct Driver {
  client: Option<Client>,
}

impl Driver {
  pub fn new(endpoint: String) {

  }

  pub async fn connect(&self) {

  }

  pub async fn close(&self) {

  }

  pub async fn insert_event<'a>(event: Event<'a>) {
  }

  pub async fn list_aggregate_events<'a>(event: Event<'a>) {
  }
}