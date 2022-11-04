use arque_common::request_generated::ListAggregateEventsRequestBody;
use super::ControllerContext;

pub fn list_aggregate_events(ctx: &ControllerContext, body: &ListAggregateEventsRequestBody) -> Result<(), Box<dyn std::error::Error>> {
  Ok(())
}