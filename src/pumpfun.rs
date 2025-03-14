use crate::{constants::SOLANA_PUBKEY, types::Swap, utils::get_now_timestamp};

use super::*;

use carbon_core::error::Error;
use carbon_pumpfun_decoder::instructions::PumpfunInstruction;

pub struct PumpFunMonitor {
    pub sender: Sender<Event>,
}

#[async_trait]
impl Processor for PumpFunMonitor {
    type InputType = InstructionProcessorInputType<PumpfunInstruction>;

    async fn process(
        &mut self,
        data: Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let signature = data.0.transaction_metadata.signature;
        let now_timestamp = get_now_timestamp();
        let (event_type, user) = match data.1.data {
            PumpfunInstruction::CreateEvent(create_event) => (
                EventType::PoolCreation {
                    mint: create_event.mint,
                    platform: SwapPlatform::PumpFun,
                },
                create_event.user,
            ),
            PumpfunInstruction::TradeEvent(trade_event) => {
                let (
                    token_in_amount,
                    token_in_decimals,
                    token_in_mint,
                    token_out_amount,
                    token_out_decimals,
                    token_out_mint,
                ) = if trade_event.is_buy {
                    (
                        trade_event.sol_amount,
                        9,
                        SOLANA_PUBKEY,
                        trade_event.token_amount,
                        6,
                        trade_event.mint,
                    )
                } else {
                    (
                        trade_event.token_amount,
                        6,
                        trade_event.mint,
                        trade_event.sol_amount,
                        9,
                        SOLANA_PUBKEY,
                    )
                };

                let (token_in_reserve, token_out_reserve) = if trade_event.is_buy {
                    (
                        trade_event.virtual_sol_reserves as f64
                            / 10_f64.powi(token_in_decimals as i32),
                        trade_event.virtual_token_reserves as f64
                            / 10_f64.powi(token_out_decimals as i32),
                    )
                } else {
                    (
                        trade_event.virtual_token_reserves as f64
                            / 10_f64.powi(token_in_decimals as i32),
                        trade_event.virtual_sol_reserves as f64
                            / 10_f64.powi(token_out_decimals as i32),
                    )
                };

                (
                    EventType::Swap(Swap {
                        token_in_amount,
                        token_in_decimals,
                        token_in_mint,
                        token_out_amount,
                        token_out_decimals,
                        token_out_mint,
                        platform: SwapPlatform::PumpFun,
                        token_in_reserve,
                        token_out_reserve,
                    }),
                    trade_event.user,
                )
            }
            _ => {
                return Ok(());
            }
        };

        self.sender
            .send(Event {
                signature,
                event_type,
                user,
                timestamp: now_timestamp,
            })
            .await
            .map_err(|error| Error::Custom(format!("send pumpfun event to receiver: {}", error)))
    }
}
