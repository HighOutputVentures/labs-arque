use async_trait::async_trait;
use rdkafka::error::KafkaError;

#[async_trait]
pub trait Stream {
    async fn send(&self, id: String, data: Vec<u8>) -> Result<(), KafkaError>;
}
