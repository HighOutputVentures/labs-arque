pub struct ListAggregateEventsParams<'a> {
    pub aggregate_id: &'a [u8],
    pub aggregate_version: Option<u32>,
    pub limit: u32,
}
