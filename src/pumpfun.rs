use crate::{types::Swap, utils::get_now_timestamp};

use super::*;

use carbon_core::error::Error;
use carbon_pumpfun_decoder::instructions::{PumpfunInstruction, trade_event::TradeEvent};

pub struct PumpFunMonitor {
    pub sender: Sender<Event>,
}

impl From<TradeEvent> for Swap {
    fn from(trade_event: TradeEvent) -> Self {
        Swap {
            mint: trade_event.mint,
            user: trade_event.user,
            is_buy: trade_event.is_buy,
            sol_amount: trade_event.sol_amount,
            token_amount: trade_event.token_amount,
            token_decimals: 6,
            platform: SwapPlatform::PumpFun,
            token_reserve: trade_event.virtual_token_reserves as f64 / 10_f64.powi(6),
            sol_reserve: trade_event.virtual_sol_reserves as f64 / 10_f64.powi(9),
        }
    }
}

#[async_trait]
impl Processor for PumpFunMonitor {
    type InputType = InstructionProcessorInputType<PumpfunInstruction>;

    async fn process(
        &mut self,
        data: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        match data.0.transaction_metadata.meta.status {
            Ok(_) => {}
            Err(_error) => {
                return Ok(());
            }
        }

        let signature = data.0.transaction_metadata.signature;
        let now_timestamp = get_now_timestamp();
        match data.1.data {
            PumpfunInstruction::TradeEvent(trade_event) => {
                let swap = Swap::from(trade_event);
                self.sender
                    .send(Event {
                        signature,
                        event_type: EventType::Swap(swap),
                        timestamp: now_timestamp,
                    })
                    .await
                    .map_err(|error| Error::Custom(format!("send event to receiver: {}", error)))?;
            }
            PumpfunInstruction::CreateEvent(create_event) => {
                self.sender
                    .send(Event {
                        signature,
                        event_type: EventType::PoolCreation {
                            mint: create_event.mint,
                            user: create_event.user,
                            platform: SwapPlatform::PumpFun,
                        },
                        timestamp: now_timestamp,
                    })
                    .await
                    .map_err(|error| Error::Custom(format!("send event to receiver: {}", error)))?;
            }
            _ => {
                return Ok(());
            }
        }

        Ok(())
    }
}
