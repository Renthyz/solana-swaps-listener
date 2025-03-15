use std::collections::HashMap;

use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};
use transactions_listener::{TransactionsListener, types::EventType};

#[tokio::main]
async fn main() {
    let layer = tracing_subscriber::fmt::layer()
        .with_line_number(true)
        .with_target(true);

    tracing_subscriber::registry()
        .with(layer)
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    // TODO: read using cli args
    let grpc_url = "https://nyc.grpc.gadflynode.com:443";
    let x_token = None;

    let grpc_urls = {
        let mut map = HashMap::new();
        map.insert(
            "example gRPC id".to_string(),
            (grpc_url.to_string(), x_token),
        );
        map
    };

    let (mut transactions_listener, mut events_receiver) =
        TransactionsListener::new(128, 255, grpc_urls).unwrap();

    transactions_listener.run().unwrap();

    loop {
        tokio::select! {
            event = events_receiver.recv() => {
                if let Some(event) = event {
                    match event.event_type {
                        EventType::Swap(swap) => {
                            info!("Received swap event: {:#?}", swap);
                        }
                        EventType::PoolCreation { .. } => {
                            info!("Received pool creation event: {:#?}", event);
                        }
                        EventType::AssociatedAccountCreation { .. } => {
                            info!("Received associated account creation event: {:#?}", event);
                        }
                    }
                }
            }
            _ = tokio::signal::ctrl_c() => {
                break;
            }
        }
    }
}
