use custom_error::custom_error;
use rocksdb::Error;

custom_error! {pub InsertEventError
  InvalidAggregateVersion = "invalid aggregate version",
  Unexpected{message:String} = "unexpected: {message}"
}

custom_error! {pub ListAggregateEventsError
  Unexpected{message:String} = "unexpected: {message}"
}

pub struct ListAggregateEventsParams<'a> {
    pub aggregate_id: &'a [u8],
    pub aggregate_version: Option<u32>,
    pub limit: usize,
}

pub struct ListAggregateEventsParamsNext<'a> {
    pub aggregate_id: &'a [u8],
    pub aggregate_version: Option<u32>,
    pub limit: Option<usize>,
}

pub struct InsertEventParams<'a> {
    pub id: &'a [u8],
    pub aggregate_id: &'a [u8],
    pub aggregate_version: u32,
    pub payload: &'a Vec<u8>,
}

pub trait Store {
    fn insert_event(&self, params: InsertEventParams) -> Result<(), InsertEventError>;
    fn list_aggregate_events(
        &self,
        params: ListAggregateEventsParams,
    ) -> Result<Vec<Vec<u8>>, Error>;
}
