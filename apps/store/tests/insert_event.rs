use rstest::*;
use arque_driver::Client;

#[rstest]
#[tokio::test]
async fn test_insert_event() {
  let client = Client::connect("tcp://localhost:4000".to_string()).await;
}

#[rstest]
#[tokio::test]
async fn test_invalid_aggregate_version() {
}
