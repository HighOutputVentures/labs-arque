mod insert_event;
mod list_aggregate_events;

use crate::store::Store;
use crate::stream::Stream;

pub use insert_event::insert_event;
pub use list_aggregate_events::list_aggregate_events;

pub struct ControllerContext {
    pub store: Box<dyn Store>,
    pub stream: Box<dyn Stream>,
}
