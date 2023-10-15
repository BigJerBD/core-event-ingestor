use crate::GoogleConfig;
use cloud_pubsub::Client;
use std::time::Duration;

pub async fn new(google_config: GoogleConfig) -> Client {
    let pubsub = match Client::new(
        google_config.application_credentials,
    )
    .await
    {
        Err(e) => panic!("Failed to initialize pubsub: {}", e),
        Ok(p) => p,
    };

    pubsub.spawn_token_renew(Duration::from_secs(60 * 10));
    pubsub
}
