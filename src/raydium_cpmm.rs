use std::str::FromStr;

use carbon_core::error::Error;
use carbon_raydium_cpmm_decoder::instructions::RaydiumCpmmInstruction;
use solana_sdk::pubkey::Pubkey;

use crate::{
    transfer::{
        extract_spl_token_info, is_transfer_check_instruction, is_transfer_instruction,
        process_transfer, process_transfer_check,
    },
    types::Swap,
    utils::get_now_timestamp,
};

use super::*;

pub struct RaydiumCpmmMonitor {
    pub sender: Sender<Event>,
    pub parsed_events: Arc<RwLock<HashSet<Event>>>,
}

#[tonic::async_trait]
impl Processor for RaydiumCpmmMonitor {
    type InputType = InstructionProcessorInputType<RaydiumCpmmInstruction>;

    async fn process(
        &mut self,
        (metadata, instruction, _nested_instructions): Self::InputType,
        _metrics: Arc<MetricsCollection>,
    ) -> CarbonResult<()> {
        let signature = metadata.transaction_metadata.signature;
        let now_timestamp = get_now_timestamp();

        let status = metadata.transaction_metadata.meta.status;
        if status.is_err() {
            return Ok(());
        }

        let event_type = match instruction.data {
            RaydiumCpmmInstruction::SwapBaseInput(_)
            | RaydiumCpmmInstruction::SwapBaseOutput(_) => {
                let input_token_account = instruction.accounts[4].pubkey.to_string();
                let output_token_account = instruction.accounts[5].pubkey.to_string();

                let post_token_balances = match metadata
                    .transaction_metadata
                    .meta
                    .post_token_balances
                    .clone()
                {
                    Some(post_token_balances) => post_token_balances,
                    None => {
                        return Err(Error::Custom("post token balances not found".to_string()));
                    }
                };

                let pre_token_balances = match metadata
                    .transaction_metadata
                    .meta
                    .pre_token_balances
                    .clone()
                {
                    Some(pre_token_balances) => pre_token_balances,
                    None => {
                        return Err(Error::Custom("pre token balances not found".to_string()));
                    }
                };

                let inner_instructions = match metadata
                    .transaction_metadata
                    .meta
                    .inner_instructions
                    .clone()
                {
                    Some(inner_instructions) => inner_instructions,
                    None => {
                        return Err(Error::Custom("inner instructions not found".to_string()));
                    }
                };

                let mut address_used = metadata
                    .transaction_metadata
                    .message
                    .static_account_keys()
                    .to_vec();

                address_used.extend(
                    metadata
                        .transaction_metadata
                        .meta
                        .loaded_addresses
                        .writable
                        .clone(),
                );

                address_used.extend(
                    metadata
                        .transaction_metadata
                        .meta
                        .loaded_addresses
                        .readonly
                        .clone(),
                );

                let spl_tokens_info = extract_spl_token_info(
                    post_token_balances.clone(),
                    pre_token_balances,
                    &address_used,
                );

                let mut transfer_data = Vec::new();
                for inner_instruction in inner_instructions {
                    for inner_instruction in inner_instruction.instructions {
                        if is_transfer_instruction(&inner_instruction.instruction, &address_used) {
                            match process_transfer(
                                &inner_instruction.instruction,
                                &address_used,
                                &spl_tokens_info,
                            ) {
                                Ok(transfer) => transfer_data.push(transfer),
                                Err(error) => {
                                    return Err(error);
                                }
                            }
                        } else if is_transfer_check_instruction(
                            &inner_instruction.instruction,
                            &address_used,
                        ) {
                            match process_transfer_check(
                                &inner_instruction.instruction,
                                &address_used,
                                &spl_tokens_info,
                            ) {
                                Ok(transfer) => transfer_data.push(transfer),
                                Err(error) => {
                                    return Err(error);
                                }
                            }
                        }
                    }
                }

                if transfer_data.is_empty() {
                    return Err(Error::Custom("transfer data not found".to_string()));
                }

                let mut pay_transfer = None;
                let mut receive_transfer = None;

                for transfer in transfer_data.clone() {
                    // Transfer in
                    if transfer.info.source == input_token_account {
                        pay_transfer = Some(transfer);
                    }
                    // Transfer out
                    else if transfer.info.destination == output_token_account {
                        receive_transfer = Some(transfer);
                    }
                }

                let pay_transfer = if let Some(pay_transfer) = pay_transfer {
                    pay_transfer
                } else {
                    return Err(Error::Custom("pay transfer not found".to_string()));
                };

                let receive_transfer = if let Some(receive_transfer) = receive_transfer {
                    receive_transfer
                } else {
                    return Err(Error::Custom("receive transfer not found".to_string()));
                };

                let mut token_in_reserve = 0;
                let mut token_out_reserve = 0;

                for post_token_balance in post_token_balances {
                    let account =
                        address_used[post_token_balance.account_index as usize].to_string();

                    if account == pay_transfer.info.destination {
                        token_in_reserve = post_token_balance
                            .ui_token_amount
                            .amount
                            .parse::<u64>()
                            .unwrap();
                    } else if account == receive_transfer.info.source {
                        token_out_reserve = post_token_balance
                            .ui_token_amount
                            .amount
                            .parse::<u64>()
                            .unwrap();
                    }
                }

                EventType::Swap(Swap {
                    token_in_amount: pay_transfer.info.amount,
                    token_in_decimals: pay_transfer.decimals,
                    token_in_mint: Pubkey::from_str(&pay_transfer.mint).unwrap(),
                    token_out_amount: receive_transfer.info.amount,
                    token_out_decimals: receive_transfer.decimals,
                    token_out_mint: Pubkey::from_str(&receive_transfer.mint).unwrap(),
                    platform: SwapPlatform::RaydiumCpmm,
                    token_in_reserve,
                    token_out_reserve,
                })
            }
            RaydiumCpmmInstruction::Initialize(_initialize) => EventType::PoolCreation {
                mint: instruction.accounts[0].pubkey,
                platform: SwapPlatform::RaydiumCpmm,
            },
            _ => {
                return Ok(());
            }
        };

        let event = Event {
            signature,
            event_type,
            user: Pubkey::from_str(&instruction.accounts[0].pubkey.to_string()).unwrap(),
            timestamp: now_timestamp,
        };

        let parsed_events_cache = self.parsed_events.read().await.clone();
        if parsed_events_cache.contains(&event) {
            return Ok(());
        }

        self.parsed_events.write().await.insert(event.clone());

        self.sender.send(event).await.map_err(|error| {
            Error::Custom(format!("send raydium cpmm event to receiver: {}", error))
        })
    }
}
