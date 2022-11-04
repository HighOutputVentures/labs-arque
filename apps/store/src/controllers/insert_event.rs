use arque_common::event_generated::Event;
use super::ControllerContext;

pub fn insert_event(ctx: &ControllerContext, body: &Event) -> Result<(), Box<dyn std::error::Error>> {
  /**
   * - call Store#insert_event
   * - forward events to Kafka
  */
  Ok(())
}
