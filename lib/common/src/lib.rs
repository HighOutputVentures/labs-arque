use event_generated::{finish_event_buffer, root_as_event, Event, EventArgs};
use flatbuffers::FlatBufferBuilder;

pub struct EventArgsType {
    pub id: Vec<u8>,
    pub type_: u16,
    pub timestamp: u32,
    pub aggregate_id: Vec<u8>,
    pub aggregate_version: u32,
    pub body: Vec<u8>,
    pub metadata: Vec<u8>,
    pub version: u8,
}

pub fn event_to_fb(event_args: EventArgsType) -> Vec<u8> {
    let mut bldr = FlatBufferBuilder::new();

    bldr.reset();

    let args = EventArgs {
        id: Some(bldr.create_vector(&event_args.id)),
        type_: event_args.type_,
        timestamp: event_args.timestamp,
        aggregate_id: Some(bldr.create_vector(&event_args.aggregate_id)),
        aggregate_version: event_args.aggregate_version,
        body: Some(bldr.create_vector(&event_args.body)),
        metadata: Some(bldr.create_vector(&event_args.metadata)),
        version: event_args.version,
    };

    let event_data = Event::create(&mut bldr, &args);

    finish_event_buffer(&mut bldr, event_data);

    return bldr.finished_data().to_owned();
}

pub fn event_to_event_args(event: Event) -> EventArgsType {
    return EventArgsType {
        id: event.id().unwrap().to_vec(),
        type_: event.type_(),
        timestamp: event.timestamp(),
        aggregate_id: event.aggregate_id().unwrap().to_vec(),
        aggregate_version: event.aggregate_version(),
        body: event.body().unwrap().to_vec(),
        metadata: event.metadata().unwrap().to_vec(),
        version: event.version(),
    };
}

pub fn fb_to_event(buf: &[u8]) -> Event {
    return root_as_event(buf).expect("failed to verify event");
}

pub mod event_generated;
