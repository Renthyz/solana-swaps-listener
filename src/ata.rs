use crate::utils::get_now_timestamp;

use super::*;

use carbon_core::error::Error;
use carbon_spl_associated_token_account_decoder::instructions::SplAssociatedTokenAccountInstruction;

pub struct PumpFunMonitor {
    pub sender: Sender<Event>,
    pub parsed_events: Arc<RwLock<HashSet<Event>>>,
}

#[tonic::async_trait]
impl Processor for PumpFunMonitor {
    type InputType = InstructionProcessorInputType<SplAssociatedTokenAccountInstruction>;

    async fn process(
        &mut self,
        (metadata, instruction, _nested_instructions): Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let signature = metadata.transaction_metadata.signature;
        let now_timestamp = get_now_timestamp();

        let idempotent = match instruction.data {
            SplAssociatedTokenAccountInstruction::Create(_) => false,
            SplAssociatedTokenAccountInstruction::CreateIdempotent(_) => true,
            _ => {
                return Ok(());
            }
        };

        let event = Event {
            signature,
            event_type: EventType::AssociatedAccountCreation {
                mint: instruction.accounts[1].pubkey,
                account: instruction.accounts[0].pubkey,
                idempotent,
            },
            user: instruction.accounts[2].pubkey,
            timestamp: now_timestamp,
        };

        let parsed_events_cache = self.parsed_events.read().await;
        if parsed_events_cache.contains(&event) {
            return Ok(());
        }
        self.parsed_events.write().await.insert(event.clone());

        self.sender
            .send(event)
            .await
            .map_err(|error| Error::Custom(format!("send pumpfun event to receiver: {}", error)))
    }
}
