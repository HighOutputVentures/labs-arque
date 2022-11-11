use super::Stream;
use async_trait::async_trait;
use rdkafka::error::KafkaError;

pub struct MockKafkaStream;

#[async_trait]
impl Stream for MockKafkaStream {
    async fn send(&self, id: String, data: Vec<u8>) -> Result<(), KafkaError> {
        println!("send");
        Ok(())
    }
}
