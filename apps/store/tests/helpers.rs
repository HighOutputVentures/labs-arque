use std::iter::repeat_with;
use arque_common::request_generated::{Event, EventArgs, InsertEventRequestBody, InsertEventRequestBodyArgs, Request, RequestArgs, RequestBody};
use flatbuffers::{WIPOffset, FlatBufferBuilder};

pub fn random_bytes(len: usize) -> Vec<u8> {
  repeat_with(|| fastrand::u8(..)).take(len).collect()
}

#[derive(Default)]
pub struct GenerateFakeEventArgs {
  id: Option<Vec<u8>>,
  type_: Option<u16>,
  aggregate_id: Option<Vec<u8>>,
  aggregate_version: Option<u32>,
  body: Option<Vec<u8>>,
  metadata: Option<Vec<u8>>,
  timestamp: Option<u32>,
}

pub fn generate_fake_event<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
  fbb: &'mut_bldr mut FlatBufferBuilder<'bldr>,
  args: &'args GenerateFakeEventArgs,
) -> WIPOffset<Event<'args>> {
  let args = EventArgs {
    id: Some(fbb.create_vector(args.id.as_ref().unwrap_or(&random_bytes(12)))),
    type_: args.type_.unwrap_or(fastrand::u16(..)),
    aggregate_id: Some(fbb.create_vector(args.aggregate_id.as_ref().unwrap_or(&random_bytes(12)))),
    aggregate_version: args.aggregate_version.unwrap_or(fastrand::u32(..)),
    body: Some(fbb.create_vector(args.body.as_ref().unwrap_or(&random_bytes(1024)))),
    metadata: Some(fbb.create_vector(args.metadata.as_ref().unwrap_or(&random_bytes(64)))),
    timestamp: args.timestamp.unwrap_or(fastrand::u32(..)),
  };

  Event::create(fbb, &args)
}

pub fn generate_fake_insert_event_request<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
  fbb: &'mut_bldr mut FlatBufferBuilder<'bldr>,
) -> WIPOffset<Request<'args>> {
  let args = GenerateFakeEventArgs::default();

  let event = generate_fake_event(fbb, &args);

  let args = InsertEventRequestBodyArgs {
    event: Some(event)
  };

  let body = InsertEventRequestBody::create(fbb, &args);

  let args = RequestArgs {
    body: Some(body.as_union_value()),
    body_type: RequestBody::InsertEvent
  };

  Request::create(fbb, &args)
}