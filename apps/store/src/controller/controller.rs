use arque_common::request_generated::{Event,ListAggregateEventsRequestBody};
use arque_common::response_generated::{Response};

pub struct Controller {}

impl Controller {
  pub fn handle_request(data: &Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    /**
     * - deserialize data
     * - determine the request type
     * - call the corresponding controller
     */
    Ok(vec![])
  }

  fn insert_event(body: &Event) -> Result<(), Box<dyn std::error::Error>> {
    /**
     * - call Store#insert_event
     * - forward events to Kafka
    */
    Ok(())
  }

  fn list_aggregate_events(body: &ListAggregateEventsRequestBody) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
  }
}
