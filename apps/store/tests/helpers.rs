use arque_common::request_generated::{EventBuilder, Event};
use flatbuffers::{FlatBufferBuilder, WIPOffset};

pub fn generate_fake_event<'a>(fbb: &mut FlatBufferBuilder<'a>) -> WIPOffset<Event<'a>> {
  let bldr = EventBuilder::new(fbb);

  bldr.finish()
}