use arque_common::request_generated::{Event, EventArgs};
use flatbuffers::{FlatBufferBuilder, WIPOffset};
use std::iter::repeat_with;

pub fn random_bytes(len: usize) -> Vec<u8> {
    repeat_with(|| fastrand::u8(..)).take(len).collect()
}

#[derive(Debug, Default, Clone)]
pub struct GenerateFakeEventArgs<'a> {
    pub id: Option<&'a [u8]>,
    pub type_: Option<u16>,
    pub aggregate_id: Option<&'a [u8]>,
    pub aggregate_version: Option<u32>,
    pub body: Option<&'a [u8]>,
    pub meta: Option<&'a [u8]>,
}

pub fn generate_fake_event<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
  fbb: &'mut_bldr mut FlatBufferBuilder<'bldr>,
  args: &'args GenerateFakeEventArgs,
) -> WIPOffset<Event<'args>> {
  let args = EventArgs {
      id: Some(fbb.create_vector(args.id.unwrap_or(&random_bytes(12)))),
      type_: args.type_.unwrap_or(fastrand::u16(..)),
      aggregate_id: Some(
          fbb.create_vector(args.aggregate_id.unwrap_or(&random_bytes(12))),
      ),
      aggregate_version: args.aggregate_version.unwrap_or(1),
      body: Some(fbb.create_vector(args.body.unwrap_or(&random_bytes(1024)))),
      meta: Some(fbb.create_vector(args.meta.unwrap_or(&random_bytes(64)))),
  };

  Event::create(fbb, &args)
}
