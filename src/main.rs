use std::collections::HashMap;

use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use transactions_listener::{SubscribeRequest, TransactionsListener};

#[tokio::main]
async fn main() {
    let layer = tracing_subscriber::fmt::layer()
        .with_line_number(true)
        .with_target(true);

    tracing_subscriber::registry()
        .with(layer)
        .with(tracing_subscriber::fmt::layer().with_line_number(true))
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    // TODO: read using cli args
    let grpc_url = "http://fra-geyser.rpc.urbanaio.com";
    let x_token = None;

    let grpc_urls = {
        let mut map = HashMap::new();
        map.insert("urban fra".to_string(), (grpc_url.to_string(), x_token));
        map
    };

    let (mut transactions_listener, mut events_receiver) = TransactionsListener::new(
        128,
        grpc_urls,
        SubscribeRequest {
            use_pumpfun: true,
            ..Default::default()
        },
    )
    .unwrap();

    transactions_listener.run().unwrap();
    while let Some(event) = events_receiver.recv().await {
        info!("Received event: {:#?}", event);
    }
}
