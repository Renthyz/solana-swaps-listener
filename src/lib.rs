use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use carbon_core::pipeline::{Pipeline, PipelineBuilder};
use carbon_pumpfun_decoder::PumpfunDecoder;
use carbon_yellowstone_grpc_datasource::YellowstoneGrpcGeyserClient;
use tokio::{
    sync::{
        RwLock,
        mpsc::{self, Receiver, Sender},
    },
    task::JoinHandle,
};
use types::Event;

use crate::constants::programs::*;
pub(crate) use {
    carbon_core::{
        error::CarbonResult, error::Error, instruction::InstructionProcessorInputType,
        metrics::MetricsCollection, processor::Processor,
    },
    tonic::async_trait,
    types::*,
};

pub(crate) mod constants;
pub mod pumpfun;
pub mod types;
pub(crate) mod utils;

pub struct TransactionsListener {
    pub sender: Sender<Event>,
    pub grpc_urls: HashMap<String, (String, Option<String>)>,
    pub subscribe_request: SubscribeRequest,
    pub pipeline_thread: Option<JoinHandle<CarbonResult<()>>>,
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

    pub fn get_pipeline_builder(&self, sender: Sender<Event>) -> PipelineBuilder {
        let mut pipeline = carbon_core::pipeline::Pipeline::builder()
            .shutdown_strategy(carbon_core::pipeline::ShutdownStrategy::Immediate);

        if self.use_pumpfun {
            pipeline = pipeline.instruction(
                PumpfunDecoder,
                pumpfun::PumpFunMonitor {
                    sender: sender.clone(),
                },
            );
        }

        pipeline
    }
}

impl TransactionsListener {
    pub fn new(
        buffer_size: usize,
        grpc_urls: HashMap<String, (String, Option<String>)>,
        subscribe_request: SubscribeRequest,
    ) -> CarbonResult<(Self, Receiver<Event>)> {
        let (sender, receiver) = mpsc::channel(buffer_size);

        Ok((
            Self {
                sender,
                grpc_urls,
                subscribe_request,
                pipeline_thread: None,
            },
            receiver,
        ))
    }

    pub fn get_pipeline(&self, sender: Sender<Event>) -> CarbonResult<Pipeline> {
        let mut pipeline = self.subscribe_request.get_pipeline_builder(sender.clone());
        for (_id, (url, x_token)) in self.grpc_urls.iter() {
            let client = YellowstoneGrpcGeyserClient::new(
                url.clone(),
                x_token.clone(),
                None,
                HashMap::new(),
                HashMap::new(),
                Arc::new(RwLock::new(HashSet::new())),
            );
            pipeline = pipeline.datasource(client);
        }

        pipeline
            .build()
            .map_err(|error| Error::Custom(format!("build pipeline: {}", error)))
    }

    pub fn get_pipeline_thread(&self) -> CarbonResult<JoinHandle<CarbonResult<()>>> {
        let mut pipeline = self.get_pipeline(self.sender.clone())?;
        Ok(tokio::spawn(async move { pipeline.run().await }))
    }

    pub fn run(&mut self) -> CarbonResult<()> {
        if self.pipeline_thread.is_some() {
            return Err(Error::Custom("pipeline thread already running".to_string()));
        }
        self.pipeline_thread = Some(self.get_pipeline_thread()?);
        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(thread) = self.pipeline_thread.take() {
            thread.abort();
        }
        self.pipeline_thread = None;
    }

    pub fn update_subscribe_request(
        &mut self,
        subscribe_request: SubscribeRequest,
    ) -> CarbonResult<()> {
        self.subscribe_request = subscribe_request;
        let new_pipeline_thread = self.get_pipeline_thread()?;
        self.stop();
        self.pipeline_thread = Some(new_pipeline_thread);

        Ok(())
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
