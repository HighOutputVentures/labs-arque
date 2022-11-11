use super::stream::Stream;

use async_trait::async_trait;

use rdkafka::{
    config::ClientConfig,
    error::KafkaError,
    producer::{FutureProducer, FutureRecord},
};

fn producer(broker: &str) -> FutureProducer {
    ClientConfig::new()
        .set("bootstrap.servers", broker)
        .create()
        .expect("Producer creation error")
}

pub struct KafkaStream {
    pub broker: String,
}

#[async_trait]
impl Stream for KafkaStream {
    async fn send(&self, id: String, data: Vec<u8>) -> Result<(), KafkaError> {
        let producer = producer(self.broker.as_str());

        let status = producer.send(
            FutureRecord::<str, Vec<u8>>::to(id.as_str()).payload(data.as_ref()),
            None,
        );

        println!("Delivered status: {:?}", status.await);

        Ok(())
    }
}

// #[tokio::main]
// async fn main() {
//     let stream = KafkaStream {
//         broker: "localhost:9092".to_string(),
//     };

//     stream
//         .send("demo".to_string(), vec![0, 1, 2])
//         .await
//         .unwrap();
// }
