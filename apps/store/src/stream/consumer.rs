use futures::StreamExt;
use rdkafka::{
    consumer::{Consumer, StreamConsumer},
    ClientConfig, Message,
};

fn consumer(group_id: &str, broker: &str) -> StreamConsumer {
    ClientConfig::new()
        .set("group.id", group_id)
        .set("bootstrap.servers", broker)
        .set("auto.offset.reset", "earliest")
        .create()
        .expect("Consumer err")
}

async fn display(c: StreamConsumer) {
    while let Some(Ok(msg)) = c.stream().next().await {
        if let Some(data) = msg.payload() {
            println!("message: {:?}", String::from_utf8(data.to_owned()))
        }
    }
}

#[tokio::main]
async fn main() {
    let topic = "demo";
    let broker = "localhost:9092";
    let group_id = "g2";

    let subscribe = true;
    let consumer = consumer(group_id, broker);
    if subscribe {
        consumer.subscribe(&[topic]).expect("Unable to subscribe");
        let _ = tokio::join!(display(consumer));
    } else {
        consumer.unsubscribe();
    }
}
