mod insert_event;
mod list_aggregate_events;

use crate::controllers::insert_event::insert_event;
use crate::controllers::list_aggregate_events::list_aggregate_events;
use crate::store::Store;
use arque_common::request_generated::{root_as_request, RequestBody};

pub struct ControllerContext {
    pub store: Box<dyn Store>,
}

pub fn handle_request(
    ctx: &ControllerContext,
    data: &Vec<u8>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let request = root_as_request(data).unwrap();

    if request.body_type() == RequestBody::InsertEvent {
        insert_event(ctx, &request.body_as_insert_event().unwrap()).unwrap();
    } else if request.body_type() == RequestBody::ListAggregateEvents {
        list_aggregate_events(ctx, &request.body_as_list_aggregate_events().unwrap()).unwrap();
    }

    Ok(vec![])
}
