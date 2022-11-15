// automatically generated by the FlatBuffers compiler, do not modify


// @generated

use core::mem;
use core::cmp::Ordering;

extern crate flatbuffers;
use self::flatbuffers::{EndianScalar, Follow};

#[deprecated(since = "2.0.0", note = "Use associated constants instead. This will no longer be generated in 2021.")]
pub const ENUM_MIN_REQUEST_BODY: u8 = 0;
#[deprecated(since = "2.0.0", note = "Use associated constants instead. This will no longer be generated in 2021.")]
pub const ENUM_MAX_REQUEST_BODY: u8 = 2;
#[deprecated(since = "2.0.0", note = "Use associated constants instead. This will no longer be generated in 2021.")]
#[allow(non_camel_case_types)]
pub const ENUM_VALUES_REQUEST_BODY: [RequestBody; 3] = [
  RequestBody::NONE,
  RequestBody::InsertEvent,
  RequestBody::ListAggregateEvents,
];

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct RequestBody(pub u8);
#[allow(non_upper_case_globals)]
impl RequestBody {
  pub const NONE: Self = Self(0);
  pub const InsertEvent: Self = Self(1);
  pub const ListAggregateEvents: Self = Self(2);

  pub const ENUM_MIN: u8 = 0;
  pub const ENUM_MAX: u8 = 2;
  pub const ENUM_VALUES: &'static [Self] = &[
    Self::NONE,
    Self::InsertEvent,
    Self::ListAggregateEvents,
  ];
  /// Returns the variant's name or "" if unknown.
  pub fn variant_name(self) -> Option<&'static str> {
    match self {
      Self::NONE => Some("NONE"),
      Self::InsertEvent => Some("InsertEvent"),
      Self::ListAggregateEvents => Some("ListAggregateEvents"),
      _ => None,
    }
  }
}
impl core::fmt::Debug for RequestBody {
  fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
    if let Some(name) = self.variant_name() {
      f.write_str(name)
    } else {
      f.write_fmt(format_args!("<UNKNOWN {:?}>", self.0))
    }
  }
}
impl<'a> flatbuffers::Follow<'a> for RequestBody {
  type Inner = Self;
  #[inline]
  fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
    let b = unsafe {
      flatbuffers::read_scalar_at::<u8>(buf, loc)
    };
    Self(b)
  }
}

impl flatbuffers::Push for RequestBody {
    type Output = RequestBody;
    #[inline]
    fn push(&self, dst: &mut [u8], _rest: &[u8]) {
        unsafe { flatbuffers::emplace_scalar::<u8>(dst, self.0); }
    }
}

impl flatbuffers::EndianScalar for RequestBody {
  #[inline]
  fn to_little_endian(self) -> Self {
    let b = u8::to_le(self.0);
    Self(b)
  }
  #[inline]
  #[allow(clippy::wrong_self_convention)]
  fn from_little_endian(self) -> Self {
    let b = u8::from_le(self.0);
    Self(b)
  }
}

impl<'a> flatbuffers::Verifiable for RequestBody {
  #[inline]
  fn run_verifier(
    v: &mut flatbuffers::Verifier, pos: usize
  ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
    use self::flatbuffers::Verifiable;
    u8::run_verifier(v, pos)
  }
}

impl flatbuffers::SimpleToVerifyInSlice for RequestBody {}
pub struct RequestBodyUnionTableOffset {}

pub enum EventOffset {}
#[derive(Copy, Clone, PartialEq)]

pub struct Event<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for Event<'a> {
  type Inner = Event<'a>;
  #[inline]
  fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
    Self { _tab: flatbuffers::Table { buf, loc } }
  }
}

impl<'a> Event<'a> {
  pub const VT_ID: flatbuffers::VOffsetT = 4;
  pub const VT_TYPE_: flatbuffers::VOffsetT = 6;
  pub const VT_AGGREGATE_ID: flatbuffers::VOffsetT = 8;
  pub const VT_AGGREGATE_VERSION: flatbuffers::VOffsetT = 10;
  pub const VT_BODY: flatbuffers::VOffsetT = 12;
  pub const VT_META: flatbuffers::VOffsetT = 14;

  #[inline]
  pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
    Event { _tab: table }
  }
  #[allow(unused_mut)]
  pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
    _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
    args: &'args EventArgs<'args>
  ) -> flatbuffers::WIPOffset<Event<'bldr>> {
    let mut builder = EventBuilder::new(_fbb);
    if let Some(x) = args.meta { builder.add_meta(x); }
    if let Some(x) = args.body { builder.add_body(x); }
    builder.add_aggregate_version(args.aggregate_version);
    if let Some(x) = args.aggregate_id { builder.add_aggregate_id(x); }
    if let Some(x) = args.id { builder.add_id(x); }
    builder.add_type_(args.type_);
    builder.finish()
  }


  #[inline]
  pub fn id(&self) -> Option<&'a [u8]> {
    self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u8>>>(Event::VT_ID, None).map(|v| v.safe_slice())
  }
  #[inline]
  pub fn type_(&self) -> u16 {
    self._tab.get::<u16>(Event::VT_TYPE_, Some(0)).unwrap()
  }
  #[inline]
  pub fn aggregate_id(&self) -> Option<&'a [u8]> {
    self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u8>>>(Event::VT_AGGREGATE_ID, None).map(|v| v.safe_slice())
  }
  #[inline]
  pub fn aggregate_version(&self) -> u32 {
    self._tab.get::<u32>(Event::VT_AGGREGATE_VERSION, Some(0)).unwrap()
  }
  #[inline]
  pub fn body(&self) -> Option<&'a [u8]> {
    self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u8>>>(Event::VT_BODY, None).map(|v| v.safe_slice())
  }
  #[inline]
  pub fn meta(&self) -> Option<&'a [u8]> {
    self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u8>>>(Event::VT_META, None).map(|v| v.safe_slice())
  }
}

impl flatbuffers::Verifiable for Event<'_> {
  #[inline]
  fn run_verifier(
    v: &mut flatbuffers::Verifier, pos: usize
  ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
    use self::flatbuffers::Verifiable;
    v.visit_table(pos)?
     .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, u8>>>("id", Self::VT_ID, false)?
     .visit_field::<u16>("type_", Self::VT_TYPE_, false)?
     .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, u8>>>("aggregate_id", Self::VT_AGGREGATE_ID, false)?
     .visit_field::<u32>("aggregate_version", Self::VT_AGGREGATE_VERSION, false)?
     .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, u8>>>("body", Self::VT_BODY, false)?
     .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, u8>>>("meta", Self::VT_META, false)?
     .finish();
    Ok(())
  }
}
pub struct EventArgs<'a> {
    pub id: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u8>>>,
    pub type_: u16,
    pub aggregate_id: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u8>>>,
    pub aggregate_version: u32,
    pub body: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u8>>>,
    pub meta: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u8>>>,
}
impl<'a> Default for EventArgs<'a> {
  #[inline]
  fn default() -> Self {
    EventArgs {
      id: None,
      type_: 0,
      aggregate_id: None,
      aggregate_version: 0,
      body: None,
      meta: None,
    }
  }
}

pub struct EventBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> EventBuilder<'a, 'b> {
  #[inline]
  pub fn add_id(&mut self, id: flatbuffers::WIPOffset<flatbuffers::Vector<'b , u8>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(Event::VT_ID, id);
  }
  #[inline]
  pub fn add_type_(&mut self, type_: u16) {
    self.fbb_.push_slot::<u16>(Event::VT_TYPE_, type_, 0);
  }
  #[inline]
  pub fn add_aggregate_id(&mut self, aggregate_id: flatbuffers::WIPOffset<flatbuffers::Vector<'b , u8>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(Event::VT_AGGREGATE_ID, aggregate_id);
  }
  #[inline]
  pub fn add_aggregate_version(&mut self, aggregate_version: u32) {
    self.fbb_.push_slot::<u32>(Event::VT_AGGREGATE_VERSION, aggregate_version, 0);
  }
  #[inline]
  pub fn add_body(&mut self, body: flatbuffers::WIPOffset<flatbuffers::Vector<'b , u8>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(Event::VT_BODY, body);
  }
  #[inline]
  pub fn add_meta(&mut self, meta: flatbuffers::WIPOffset<flatbuffers::Vector<'b , u8>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(Event::VT_META, meta);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> EventBuilder<'a, 'b> {
    let start = _fbb.start_table();
    EventBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<Event<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

impl core::fmt::Debug for Event<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let mut ds = f.debug_struct("Event");
      ds.field("id", &self.id());
      ds.field("type_", &self.type_());
      ds.field("aggregate_id", &self.aggregate_id());
      ds.field("aggregate_version", &self.aggregate_version());
      ds.field("body", &self.body());
      ds.field("meta", &self.meta());
      ds.finish()
  }
}
pub enum InsertEventRequestBodyOffset {}
#[derive(Copy, Clone, PartialEq)]

pub struct InsertEventRequestBody<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for InsertEventRequestBody<'a> {
  type Inner = InsertEventRequestBody<'a>;
  #[inline]
  fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
    Self { _tab: flatbuffers::Table { buf, loc } }
  }
}

impl<'a> InsertEventRequestBody<'a> {
  pub const VT_EVENT: flatbuffers::VOffsetT = 4;

  #[inline]
  pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
    InsertEventRequestBody { _tab: table }
  }
  #[allow(unused_mut)]
  pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
    _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
    args: &'args InsertEventRequestBodyArgs<'args>
  ) -> flatbuffers::WIPOffset<InsertEventRequestBody<'bldr>> {
    let mut builder = InsertEventRequestBodyBuilder::new(_fbb);
    if let Some(x) = args.event { builder.add_event(x); }
    builder.finish()
  }


  #[inline]
  pub fn event(&self) -> Option<Event<'a>> {
    self._tab.get::<flatbuffers::ForwardsUOffset<Event>>(InsertEventRequestBody::VT_EVENT, None)
  }
}

impl flatbuffers::Verifiable for InsertEventRequestBody<'_> {
  #[inline]
  fn run_verifier(
    v: &mut flatbuffers::Verifier, pos: usize
  ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
    use self::flatbuffers::Verifiable;
    v.visit_table(pos)?
     .visit_field::<flatbuffers::ForwardsUOffset<Event>>("event", Self::VT_EVENT, false)?
     .finish();
    Ok(())
  }
}
pub struct InsertEventRequestBodyArgs<'a> {
    pub event: Option<flatbuffers::WIPOffset<Event<'a>>>,
}
impl<'a> Default for InsertEventRequestBodyArgs<'a> {
  #[inline]
  fn default() -> Self {
    InsertEventRequestBodyArgs {
      event: None,
    }
  }
}

pub struct InsertEventRequestBodyBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> InsertEventRequestBodyBuilder<'a, 'b> {
  #[inline]
  pub fn add_event(&mut self, event: flatbuffers::WIPOffset<Event<'b >>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<Event>>(InsertEventRequestBody::VT_EVENT, event);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> InsertEventRequestBodyBuilder<'a, 'b> {
    let start = _fbb.start_table();
    InsertEventRequestBodyBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<InsertEventRequestBody<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

impl core::fmt::Debug for InsertEventRequestBody<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let mut ds = f.debug_struct("InsertEventRequestBody");
      ds.field("event", &self.event());
      ds.finish()
  }
}
pub enum ListAggregateEventsRequestBodyOffset {}
#[derive(Copy, Clone, PartialEq)]

pub struct ListAggregateEventsRequestBody<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for ListAggregateEventsRequestBody<'a> {
  type Inner = ListAggregateEventsRequestBody<'a>;
  #[inline]
  fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
    Self { _tab: flatbuffers::Table { buf, loc } }
  }
}

impl<'a> ListAggregateEventsRequestBody<'a> {
  pub const VT_AGGREGATE_ID: flatbuffers::VOffsetT = 4;
  pub const VT_AGGREGATE_VERSION: flatbuffers::VOffsetT = 6;
  pub const VT_LIMIT: flatbuffers::VOffsetT = 8;

  #[inline]
  pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
    ListAggregateEventsRequestBody { _tab: table }
  }
  #[allow(unused_mut)]
  pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
    _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
    args: &'args ListAggregateEventsRequestBodyArgs<'args>
  ) -> flatbuffers::WIPOffset<ListAggregateEventsRequestBody<'bldr>> {
    let mut builder = ListAggregateEventsRequestBodyBuilder::new(_fbb);
    builder.add_limit(args.limit);
    builder.add_aggregate_version(args.aggregate_version);
    if let Some(x) = args.aggregate_id { builder.add_aggregate_id(x); }
    builder.finish()
  }


  #[inline]
  pub fn aggregate_id(&self) -> Option<&'a [u8]> {
    self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u8>>>(ListAggregateEventsRequestBody::VT_AGGREGATE_ID, None).map(|v| v.safe_slice())
  }
  #[inline]
  pub fn aggregate_version(&self) -> u32 {
    self._tab.get::<u32>(ListAggregateEventsRequestBody::VT_AGGREGATE_VERSION, Some(0)).unwrap()
  }
  #[inline]
  pub fn limit(&self) -> u32 {
    self._tab.get::<u32>(ListAggregateEventsRequestBody::VT_LIMIT, Some(0)).unwrap()
  }
}

impl flatbuffers::Verifiable for ListAggregateEventsRequestBody<'_> {
  #[inline]
  fn run_verifier(
    v: &mut flatbuffers::Verifier, pos: usize
  ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
    use self::flatbuffers::Verifiable;
    v.visit_table(pos)?
     .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, u8>>>("aggregate_id", Self::VT_AGGREGATE_ID, false)?
     .visit_field::<u32>("aggregate_version", Self::VT_AGGREGATE_VERSION, false)?
     .visit_field::<u32>("limit", Self::VT_LIMIT, false)?
     .finish();
    Ok(())
  }
}
pub struct ListAggregateEventsRequestBodyArgs<'a> {
    pub aggregate_id: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u8>>>,
    pub aggregate_version: u32,
    pub limit: u32,
}
impl<'a> Default for ListAggregateEventsRequestBodyArgs<'a> {
  #[inline]
  fn default() -> Self {
    ListAggregateEventsRequestBodyArgs {
      aggregate_id: None,
      aggregate_version: 0,
      limit: 0,
    }
  }
}

pub struct ListAggregateEventsRequestBodyBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> ListAggregateEventsRequestBodyBuilder<'a, 'b> {
  #[inline]
  pub fn add_aggregate_id(&mut self, aggregate_id: flatbuffers::WIPOffset<flatbuffers::Vector<'b , u8>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(ListAggregateEventsRequestBody::VT_AGGREGATE_ID, aggregate_id);
  }
  #[inline]
  pub fn add_aggregate_version(&mut self, aggregate_version: u32) {
    self.fbb_.push_slot::<u32>(ListAggregateEventsRequestBody::VT_AGGREGATE_VERSION, aggregate_version, 0);
  }
  #[inline]
  pub fn add_limit(&mut self, limit: u32) {
    self.fbb_.push_slot::<u32>(ListAggregateEventsRequestBody::VT_LIMIT, limit, 0);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> ListAggregateEventsRequestBodyBuilder<'a, 'b> {
    let start = _fbb.start_table();
    ListAggregateEventsRequestBodyBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<ListAggregateEventsRequestBody<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

impl core::fmt::Debug for ListAggregateEventsRequestBody<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let mut ds = f.debug_struct("ListAggregateEventsRequestBody");
      ds.field("aggregate_id", &self.aggregate_id());
      ds.field("aggregate_version", &self.aggregate_version());
      ds.field("limit", &self.limit());
      ds.finish()
  }
}
pub enum RequestOffset {}
#[derive(Copy, Clone, PartialEq)]

pub struct Request<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for Request<'a> {
  type Inner = Request<'a>;
  #[inline]
  fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
    Self { _tab: flatbuffers::Table { buf, loc } }
  }
}

impl<'a> Request<'a> {
  pub const VT_BODY_TYPE: flatbuffers::VOffsetT = 4;
  pub const VT_BODY: flatbuffers::VOffsetT = 6;

  #[inline]
  pub fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
    Request { _tab: table }
  }
  #[allow(unused_mut)]
  pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
    _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
    args: &'args RequestArgs
  ) -> flatbuffers::WIPOffset<Request<'bldr>> {
    let mut builder = RequestBuilder::new(_fbb);
    if let Some(x) = args.body { builder.add_body(x); }
    builder.add_body_type(args.body_type);
    builder.finish()
  }


  #[inline]
  pub fn body_type(&self) -> RequestBody {
    self._tab.get::<RequestBody>(Request::VT_BODY_TYPE, Some(RequestBody::NONE)).unwrap()
  }
  #[inline]
  pub fn body(&self) -> Option<flatbuffers::Table<'a>> {
    self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Table<'a>>>(Request::VT_BODY, None)
  }
  #[inline]
  #[allow(non_snake_case)]
  pub fn body_as_insert_event(&self) -> Option<InsertEventRequestBody<'a>> {
    if self.body_type() == RequestBody::InsertEvent {
      self.body().map(InsertEventRequestBody::init_from_table)
    } else {
      None
    }
  }

  #[inline]
  #[allow(non_snake_case)]
  pub fn body_as_list_aggregate_events(&self) -> Option<ListAggregateEventsRequestBody<'a>> {
    if self.body_type() == RequestBody::ListAggregateEvents {
      self.body().map(ListAggregateEventsRequestBody::init_from_table)
    } else {
      None
    }
  }

}

impl flatbuffers::Verifiable for Request<'_> {
  #[inline]
  fn run_verifier(
    v: &mut flatbuffers::Verifier, pos: usize
  ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
    use self::flatbuffers::Verifiable;
    v.visit_table(pos)?
     .visit_union::<RequestBody, _>("body_type", Self::VT_BODY_TYPE, "body", Self::VT_BODY, false, |key, v, pos| {
        match key {
          RequestBody::InsertEvent => v.verify_union_variant::<flatbuffers::ForwardsUOffset<InsertEventRequestBody>>("RequestBody::InsertEvent", pos),
          RequestBody::ListAggregateEvents => v.verify_union_variant::<flatbuffers::ForwardsUOffset<ListAggregateEventsRequestBody>>("RequestBody::ListAggregateEvents", pos),
          _ => Ok(()),
        }
     })?
     .finish();
    Ok(())
  }
}
pub struct RequestArgs {
    pub body_type: RequestBody,
    pub body: Option<flatbuffers::WIPOffset<flatbuffers::UnionWIPOffset>>,
}
impl<'a> Default for RequestArgs {
  #[inline]
  fn default() -> Self {
    RequestArgs {
      body_type: RequestBody::NONE,
      body: None,
    }
  }
}

pub struct RequestBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> RequestBuilder<'a, 'b> {
  #[inline]
  pub fn add_body_type(&mut self, body_type: RequestBody) {
    self.fbb_.push_slot::<RequestBody>(Request::VT_BODY_TYPE, body_type, RequestBody::NONE);
  }
  #[inline]
  pub fn add_body(&mut self, body: flatbuffers::WIPOffset<flatbuffers::UnionWIPOffset>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(Request::VT_BODY, body);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> RequestBuilder<'a, 'b> {
    let start = _fbb.start_table();
    RequestBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<Request<'a>> {
    let o = self.fbb_.end_table(self.start_);
    flatbuffers::WIPOffset::new(o.value())
  }
}

impl core::fmt::Debug for Request<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let mut ds = f.debug_struct("Request");
      ds.field("body_type", &self.body_type());
      match self.body_type() {
        RequestBody::InsertEvent => {
          if let Some(x) = self.body_as_insert_event() {
            ds.field("body", &x)
          } else {
            ds.field("body", &"InvalidFlatbuffer: Union discriminant does not match value.")
          }
        },
        RequestBody::ListAggregateEvents => {
          if let Some(x) = self.body_as_list_aggregate_events() {
            ds.field("body", &x)
          } else {
            ds.field("body", &"InvalidFlatbuffer: Union discriminant does not match value.")
          }
        },
        _ => {
          let x: Option<()> = None;
          ds.field("body", &x)
        },
      };
      ds.finish()
  }
}
#[inline]
#[deprecated(since="2.0.0", note="Deprecated in favor of `root_as...` methods.")]
pub fn get_root_as_request<'a>(buf: &'a [u8]) -> Request<'a> {
  unsafe { flatbuffers::root_unchecked::<Request<'a>>(buf) }
}

#[inline]
#[deprecated(since="2.0.0", note="Deprecated in favor of `root_as...` methods.")]
pub fn get_size_prefixed_root_as_request<'a>(buf: &'a [u8]) -> Request<'a> {
  unsafe { flatbuffers::size_prefixed_root_unchecked::<Request<'a>>(buf) }
}

#[inline]
/// Verifies that a buffer of bytes contains a `Request`
/// and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_request_unchecked`.
pub fn root_as_request(buf: &[u8]) -> Result<Request, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::root::<Request>(buf)
}
#[inline]
/// Verifies that a buffer of bytes contains a size prefixed
/// `Request` and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `size_prefixed_root_as_request_unchecked`.
pub fn size_prefixed_root_as_request(buf: &[u8]) -> Result<Request, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::size_prefixed_root::<Request>(buf)
}
#[inline]
/// Verifies, with the given options, that a buffer of bytes
/// contains a `Request` and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_request_unchecked`.
pub fn root_as_request_with_opts<'b, 'o>(
  opts: &'o flatbuffers::VerifierOptions,
  buf: &'b [u8],
) -> Result<Request<'b>, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::root_with_opts::<Request<'b>>(opts, buf)
}
#[inline]
/// Verifies, with the given verifier options, that a buffer of
/// bytes contains a size prefixed `Request` and returns
/// it. Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_request_unchecked`.
pub fn size_prefixed_root_as_request_with_opts<'b, 'o>(
  opts: &'o flatbuffers::VerifierOptions,
  buf: &'b [u8],
) -> Result<Request<'b>, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::size_prefixed_root_with_opts::<Request<'b>>(opts, buf)
}
#[inline]
/// Assumes, without verification, that a buffer of bytes contains a Request and returns it.
/// # Safety
/// Callers must trust the given bytes do indeed contain a valid `Request`.
pub unsafe fn root_as_request_unchecked(buf: &[u8]) -> Request {
  flatbuffers::root_unchecked::<Request>(buf)
}
#[inline]
/// Assumes, without verification, that a buffer of bytes contains a size prefixed Request and returns it.
/// # Safety
/// Callers must trust the given bytes do indeed contain a valid size prefixed `Request`.
pub unsafe fn size_prefixed_root_as_request_unchecked(buf: &[u8]) -> Request {
  flatbuffers::size_prefixed_root_unchecked::<Request>(buf)
}
#[inline]
pub fn finish_request_buffer<'a, 'b>(
    fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    root: flatbuffers::WIPOffset<Request<'a>>) {
  fbb.finish(root, None);
}

#[inline]
pub fn finish_size_prefixed_request_buffer<'a, 'b>(fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, root: flatbuffers::WIPOffset<Request<'a>>) {
  fbb.finish_size_prefixed(root, None);
}
