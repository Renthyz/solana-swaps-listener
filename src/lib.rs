use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use carbon_core::pipeline::Pipeline;
use carbon_pumpfun_decoder::PumpfunDecoder;
use carbon_raydium_cpmm_decoder::RaydiumCpmmDecoder;
use carbon_yellowstone_grpc_datasource::YellowstoneGrpcGeyserClient;
use strum::IntoEnumIterator;
use tokio::{
    sync::{
        RwLock,
        mpsc::{self, Receiver, Sender},
    },
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
use types::Event;
use yellowstone_grpc_proto::geyser::SubscribeRequestFilterTransactions;

use crate::{constants::programs::*, prelude::*, types::*};

pub mod ata;
pub(crate) mod constants;
pub(crate) mod prelude;
pub mod pumpfun;
pub mod raydium_cpmm;
pub mod transfer;
pub mod types;
pub(crate) mod utils;

pub struct TransactionsListener {
    pub sender: Sender<Event>,
    pub grpc_urls: HashMap<String, (String, Option<String>)>,
    pub pipeline_thread: Option<(CancellationToken, JoinHandle<CarbonResult<()>>)>,
    pub events_cache: HashMap<String, Arc<RwLock<HashSet<Event>>>>,
}

impl TransactionsListener {
    pub fn new(
        buffer_size: usize,
        cache_capacity: usize,
        grpc_urls: HashMap<String, (String, Option<String>)>,
    ) -> CarbonResult<(Self, Receiver<Event>)> {
        let (sender, receiver) = mpsc::channel(buffer_size);

        let mut events_cache = HashMap::new();
        for platform in SwapPlatform::iter() {
            events_cache.insert(
                platform.to_string(),
                Arc::new(RwLock::new(HashSet::with_capacity(cache_capacity))),
            );
        }

        Ok((
            Self {
                sender,
                grpc_urls,
                pipeline_thread: None,
                events_cache,
            },
            receiver,
        ))
    }

    pub fn get_pipeline(
        &self,
        sender: Sender<Event>,
    ) -> CarbonResult<(CancellationToken, Pipeline)> {
        let cancellation_token = CancellationToken::new();
        let mut pipeline = carbon_core::pipeline::Pipeline::builder()
            .datasource_cancellation_token(cancellation_token.clone())
            .shutdown_strategy(carbon_core::pipeline::ShutdownStrategy::Immediate)
            .instruction(
                PumpfunDecoder,
                pumpfun::PumpFunMonitor {
                    sender: sender.clone(),
                    parsed_events: self
                        .events_cache
                        .get(&SwapPlatform::PumpFun.to_string())
                        .unwrap()
                        .clone(),
                },
            )
            .instruction(
                RaydiumCpmmDecoder,
                raydium_cpmm::RaydiumCpmmMonitor {
                    sender: sender.clone(),
                    parsed_events: self
                        .events_cache
                        .get(&SwapPlatform::RaydiumCpmm.to_string())
                        .unwrap()
                        .clone(),
                },
            );

        for (_id, (url, x_token)) in self.grpc_urls.iter() {
            let client = YellowstoneGrpcGeyserClient::new(
                url.clone(),
                x_token.clone(),
                None,
                HashMap::new(),
                {
                    let mut map = HashMap::new();
                    map.insert(
                        "subscribe_transactions".to_string(),
                        SubscribeRequestFilterTransactions {
                            vote: Some(false),
                            failed: None,
                            signature: None,
                            account_include: vec![
                                PUMPFUN.to_string(),
                                RAYDIUM.to_string(),
                                RAYDIUM_CPMM.to_string(),
                                RAYDIUM_CLMM.to_string(),
                            ],
                            account_exclude: vec![],
                            account_required: vec![],
                        },
                    );

                    map
                },
                Arc::new(RwLock::new(HashSet::new())),
            );

            pipeline = pipeline.datasource(client);
        }

        Ok((
            cancellation_token,
            pipeline
                .build()
                .map_err(|error| Error::Custom(format!("build pipeline: {}", error)))?,
        ))
    }

    pub fn get_pipeline_thread(
        &self,
    ) -> CarbonResult<(CancellationToken, JoinHandle<CarbonResult<()>>)> {
        let (cancellation_token, mut pipeline) = self.get_pipeline(self.sender.clone())?;
        Ok((
            cancellation_token,
            tokio::spawn(async move { pipeline.run().await }),
        ))
    }

    pub fn run(&mut self) -> CarbonResult<()> {
        if self.pipeline_thread.is_some() {
            return Err(Error::Custom("pipeline thread already running".to_string()));
        }

        self.pipeline_thread = Some(self.get_pipeline_thread()?);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some((cancellation_token, thread)) = self.pipeline_thread.take() {
            cancellation_token.cancel();
            thread.abort();
        }

        self.pipeline_thread = None;
    }

    pub fn delete_grpc_url(&mut self, id: String) -> CarbonResult<()> {
        self.grpc_urls.remove(&id);
        let new_pipeline_thread = self.get_pipeline_thread()?;
        self.stop();
        self.pipeline_thread = Some(new_pipeline_thread);

        Ok(())
    }

    pub fn add_grpc_url(
        &mut self,
        id: String,
        url: String,
        x_token: Option<String>,
    ) -> CarbonResult<()> {
        self.grpc_urls.insert(id, (url, x_token));
        let new_pipeline_thread = self.get_pipeline_thread()?;
        self.stop();
        self.pipeline_thread = Some(new_pipeline_thread);

        Ok(())
    }
}
