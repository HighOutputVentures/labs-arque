use arque_common::{EventId, event_generated::Event};

pub struct ArqueEvent {
    buf: Vec<u8>,
}

impl ArqueEvent {
    pub fn from_bytes(buf: Vec<u8>) -> Self {
        let event = arque_common::event_generated::root_as_event(&buf).unwrap();
        Self { buf }
    }
}
