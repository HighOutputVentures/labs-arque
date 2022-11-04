

use arque_common::request_generated::{Event,ListAggregateEventsRequestBody, ListAggregateEventsRequestBodyArgs, get_root_as_request, EventArgs, RequestArgs,Request, RequestBody};
use arque_common::response_generated::{Response};
use crate::store::Store;
pub struct Controller {}

impl Controller {
  pub fn handle_request(&self, data: &Vec<u8>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    /**
     * - deserialize data
     * - determine the request type
     * - call the corresponding controller
     */

    let request = get_root_as_request(data);

    if request.body_type() == RequestBody::InsertEvent {
      self.insert_event(&request.body_as_insert_event().unwrap()).unwrap();
    }
    else if request.body_type() == RequestBody::ListAggregateEvents {
      self.list_aggregate_events(&request.body_as_list_aggregate_events().unwrap()).unwrap();
    }

    Ok(vec![])
  }

  fn insert_event(&self, body: &Event) -> Result<(), Box<dyn std::error::Error>> {
    /**
     * - call Store#insert_event
     * - forward events to Kafka
    */
    println!("called insert_event");
    Ok(())
  }

  fn list_aggregate_events(&self, body: &ListAggregateEventsRequestBody) -> Result<(), Box<dyn std::error::Error>> {
    println!("called list_aggregate_events");
    Ok(())
  }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use flatbuffers::FlatBufferBuilder;
    use chrono::Local;
    use uuid::Uuid;

    #[rstest]
    #[tokio::test]
    async fn handle_insert_event_request_test(
    ) {


      let mut bldr = FlatBufferBuilder::new();

      bldr.reset();

      let id = Uuid::new_v4();
      let aggregate_id = Uuid::new_v4();

      let insert_event_args = EventArgs {
          id: Some(bldr.create_vector(&id.as_bytes().as_slice())),
          type_: 1u16,
          aggregate_id: Some(bldr.create_vector(&aggregate_id.as_bytes().as_slice())),
          aggregate_version: 1u32,
          body: Some(bldr.create_vector(&Uuid::new_v4().as_bytes().as_slice())),
          metadata: Some(bldr.create_vector(&Uuid::new_v4().as_bytes().as_slice())),
          timestamp: Local::now().timestamp() as u32,
      };
  

      let insert_event_data = Event::create(&mut bldr, &insert_event_args);

      let request_args = RequestArgs {
        body: Some(insert_event_data.as_union_value()),
        body_type: RequestBody::InsertEvent
      };
  

      let request_data = Request::create(&mut bldr, &request_args);

      bldr.finish(request_data, None);
  
      let data = bldr.finished_data();  

      let controller = Controller {};
      controller.handle_request(&data.to_vec()).unwrap();

    }

    #[rstest]
    #[tokio::test]
    async fn handle_list_aggregate_events_request_test(
    ) {

      let mut bldr = FlatBufferBuilder::new();

      bldr.reset();

      let aggregate_id = Uuid::new_v4();

      let list_aggregate_events_args = ListAggregateEventsRequestBodyArgs {
          aggregate_id: Some(bldr.create_vector(&aggregate_id.as_bytes().as_slice())),
          aggregate_version: 1u32,
          limit: 1u32
      };
  

      let list_aggregate_events_data = ListAggregateEventsRequestBody::create(&mut bldr, &list_aggregate_events_args);

      let request_args = RequestArgs {
        body: Some(list_aggregate_events_data.as_union_value()),
        body_type: RequestBody::ListAggregateEvents
      };

      let request_data = Request::create(&mut bldr, &request_args);

      bldr.finish(request_data, None);
  
      let data = bldr.finished_data();  

      let controller = Controller {};
      controller.handle_request(&data.to_vec()).unwrap();

    }

}
