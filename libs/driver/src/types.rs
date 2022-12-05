pub struct ListAggregateEventsParams {
    pub aggregate_id: Vec<u8>,
    pub aggregate_version: Option<u32>,
    pub limit: u32,
}
