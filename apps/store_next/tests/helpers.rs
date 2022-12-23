use arque_common::{
    event_generated::{Event, EventArgs, root_as_event},
    EventId,
};
use flatbuffers::{FlatBufferBuilder};
use std::{
    iter::repeat_with,
    time::{SystemTime, UNIX_EPOCH},
};

pub fn random_bytes(len: usize) -> Vec<u8> {
    repeat_with(|| fastrand::u8(..)).take(len).collect()
}

#[derive(Debug, Clone)]
pub struct GenerateFakeEventArgs {
    pub id: EventId,
    pub type_: u16,
    pub aggregate_id: [u8; 12],
    pub aggregate_version: u32,
    pub body: Vec<u8>,
    pub meta: Vec<u8>,
    pub timestamp: u32,
}

impl Default for GenerateFakeEventArgs {
    fn default() -> Self {
        Self {
            id: EventId::new(),
            type_: fastrand::u16(..),
            aggregate_id: random_bytes(12).try_into().unwrap(),
            aggregate_version: 1,
            body: random_bytes(1024),
            meta: random_bytes(64),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32
                % 0xFFFFFFFF,
        }
    }
}

pub fn generate_fake_event<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr: 'args>(
    fbb: &'mut_bldr mut FlatBufferBuilder<'bldr>,
    args: &'args GenerateFakeEventArgs,
) -> Event<'args> {
    let args = EventArgs {
        id: Some(fbb.create_vector(args.id.to_bytes())),
        type_: args.type_,
        aggregate_id: Some(fbb.create_vector(&args.aggregate_id)),
        aggregate_version: args.aggregate_version,
        body: Some(fbb.create_vector(&args.body)),
        meta: Some(fbb.create_vector(&args.meta)),
        timestamp: args.timestamp,
    };

    let event = Event::create(fbb, &args);

    fbb.finish(event, None);

    root_as_event(fbb.finished_data()).unwrap()
}
