use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use carbon_core::pipeline::Pipeline;
use carbon_pumpfun_decoder::PumpfunDecoder;
use tokio::sync::{
    RwLock,
    mpsc::{self, Receiver, Sender},
};
use types::Event;
use yellowstone_grpc_proto::geyser::{CommitmentLevel, SubscribeRequestFilterTransactions};

use crate::programs::*;
pub(crate) use {
    carbon_core::{
        error::CarbonResult, instruction::InstructionProcessorInputType,
        metrics::MetricsCollection, processor::Processor,
    },
    tonic::async_trait,
    types::*,
};

pub mod programs;
pub mod pumpfun;
pub mod types;
pub(crate) mod utils;

pub struct TransactionsListener {
    pub receiver: Receiver<Event>,
    pub sender: Sender<Event>,
    pub pipeline: Pipeline,
}

#[derive(Debug, Default)]
pub struct SubscribeRequest {
    pub use_pumpfun: bool,
    pub use_raydium: bool,
    pub use_raydium_cpmm: bool,
    pub use_raydium_clmm: bool,
}

impl SubscribeRequest {
    pub fn get_accounts_to_include(&self) -> Vec<String> {
        let mut accounts_to_include = vec![];
        if self.use_pumpfun {
            accounts_to_include.push(PUMPFUN_PROGRAM_ID.to_string());
        }
        if self.use_raydium {
            accounts_to_include.push(RAYDIUM_PROGRAM_ID.to_string());
        }
        if self.use_raydium_cpmm {
            accounts_to_include.push(RAYDIUM_CPMM_PROGRAM_ID.to_string());
        }
        if self.use_raydium_clmm {
            accounts_to_include.push(RAYDIUM_CLMM_PROGRAM_ID.to_string());
        }

        accounts_to_include
    }
}

impl TransactionsListener {
    pub fn new(
        buffer_size: usize,
        grpc_url: String,
        x_token: Option<String>,
        subscribe_request: SubscribeRequest,
    ) -> Self {
        let (sender, receiver) = mpsc::channel(buffer_size);
        let yellow_stone = carbon_yellowstone_grpc_datasource::YellowstoneGrpcGeyserClient::new(
            grpc_url.to_string(),
            x_token,
            Some(CommitmentLevel::Confirmed),
            HashMap::new(),
            {
                let mut map = HashMap::new();
                map.insert(
                    "subscribe_transactions".to_string(),
                    SubscribeRequestFilterTransactions {
                        vote: Some(false),
                        failed: None,
                        signature: None,
                        account_include: subscribe_request.get_accounts_to_include(),
                        account_exclude: vec![],
                        account_required: vec![],
                    },
                );

                map
            },
            Arc::new(RwLock::new(HashSet::new())),
        );
        let pipeline = carbon_core::pipeline::Pipeline::builder()
            .datasource(yellow_stone)
            .instruction(
                PumpfunDecoder,
                pumpfun::PumpFunMonitor {
                    sender: sender.clone(),
                },
            )
            .shutdown_strategy(carbon_core::pipeline::ShutdownStrategy::Immediate)
            .build()
            .unwrap_or_else(|_| panic!("Failed to build pipeline"));

        Self {
            sender,
            receiver,
            pipeline,
        }
    }

    pub async fn run(&mut self) -> CarbonResult<()> {
        self.pipeline.run().await
    }
}
