mod client;

pub use client::Client;

pub struct Driver {
  client: Option<Client>;
}

impl Driver {
  pub new(endpoint: String) -> Self {

  }

  pub async connect(&self) {

  }

  pub async close(&self) {

  }

  pub async insert_event(event: Event) {
  }

  pub async list_aggregate_events(event: Event) {
  }
}