use std::collections::HashMap;

use solana_account_decoder::parse_token::UiTokenAmount;
use solana_sdk::{instruction::CompiledInstruction, pubkey::Pubkey};
use solana_transaction_status::TransactionTokenBalance;

use crate::{
    constants::{SOLANA_PUBKEY, programs},
    prelude::*,
};

#[derive(Debug, Clone)]
pub struct TransferInfo {
    pub amount: u64,
    pub authority: String,
    pub destination: String,
    pub source: String,
}

#[derive(Debug, Clone)]
pub struct TransferData {
    pub info: TransferInfo,
    pub mint: String,
    pub decimals: u8,
}

pub struct TokenInfo {
    pub mint: String,
    pub amount: UiTokenAmount,
}

pub fn is_transfer_instruction(instruction: &CompiledInstruction, accounts: &[Pubkey]) -> bool {
    if accounts[instruction.program_id_index as usize] != programs::TOKEN {
        return false;
    }

    if instruction.accounts.len() < 3 || instruction.data.len() < 9 {
        return false;
    }

    if instruction.data[0] != 3 {
        return false;
    }

    for i in 0..3 {
        if instruction.accounts[i] >= accounts.len() as u8 {
            return false;
        }
    }

    true
}

pub fn is_transfer_check_instruction(
    instruction: &CompiledInstruction,
    accounts: &[Pubkey],
) -> bool {
    if accounts[instruction.program_id_index as usize] != programs::TOKEN
        && accounts[instruction.program_id_index as usize] != programs::TOKEN_2022
    {
        return false;
    }

    if instruction.accounts.len() < 4 || instruction.data.len() < 9 {
        return false;
    }

    if instruction.data[0] != 12 {
        return false;
    }

    for i in 0..4 {
        if instruction.accounts[i] >= accounts.len() as u8 {
            return false;
        }
    }

    true
}

pub fn extract_spl_token_info(
    post_token_balances: Vec<TransactionTokenBalance>,
    pre_token_balances: Vec<TransactionTokenBalance>,
    accounts: &[Pubkey],
) -> HashMap<String, TokenInfo> {
    let mut token_info_map = HashMap::new();

    for account_info in post_token_balances {
        // account_info.ui_token_amount.
        token_info_map.insert(
            accounts[account_info.account_index as usize].to_string(),
            TokenInfo {
                mint: account_info.mint,
                amount: account_info.ui_token_amount,
            },
        );
    }

    for account_info in pre_token_balances {
        token_info_map.insert(
            accounts[account_info.account_index as usize].to_string(),
            TokenInfo {
                mint: account_info.mint,
                amount: account_info.ui_token_amount,
            },
        );
    }

    token_info_map
}

pub fn process_transfer(
    instruction: &CompiledInstruction,
    accounts: &[Pubkey],
    tokens_info: &HashMap<String, TokenInfo>,
) -> CarbonResult<TransferData> {
    let amount = u64::from_le_bytes(
        instruction.data[1..9]
            .try_into()
            .map_err(|_| Error::Custom("parse transfer amount".to_string()))?,
    );

    let token_info = if let Some(token_info) =
        tokens_info.get(&accounts[instruction.accounts[1] as usize].to_string())
    {
        token_info
    } else {
        &TokenInfo {
            mint: SOLANA_PUBKEY.to_string(),
            amount: UiTokenAmount {
                ui_amount: Some(0.0),
                decimals: 9,
                amount: "0".to_string(),
                ui_amount_string: "0".to_string(),
            },
        }
    };

    let mut transfer_data = TransferData {
        info: TransferInfo {
            amount,
            authority: accounts[instruction.accounts[2] as usize].to_string(),
            destination: accounts[instruction.accounts[1] as usize].to_string(),
            source: accounts[instruction.accounts[0] as usize].to_string(),
        },
        mint: token_info.mint.clone(),
        decimals: token_info.amount.decimals,
    };

    if transfer_data.mint.is_empty() {
        transfer_data.mint = "Unknown".to_string();
    }

    Ok(transfer_data)
}

pub fn process_transfer_check(
    instruction: &CompiledInstruction,
    accounts: &[Pubkey],
    tokens_info: &HashMap<String, TokenInfo>,
) -> CarbonResult<TransferData> {
    let amount = u64::from_le_bytes(
        instruction.data[1..9]
            .try_into()
            .map_err(|_| Error::Custom("parse transfer amount".to_string()))?,
    );

    let token_info = if let Some(token_info) =
        tokens_info.get(&accounts[instruction.accounts[2] as usize].to_string())
    {
        token_info
    } else {
        &TokenInfo {
            mint: SOLANA_PUBKEY.to_string(),
            amount: UiTokenAmount {
                ui_amount: Some(0.0),
                decimals: 9,
                amount: "0".to_string(),
                ui_amount_string: "0".to_string(),
            },
        }
    };

    let mut transfer_data = TransferData {
        info: TransferInfo {
            amount,
            authority: accounts[instruction.accounts[3] as usize].to_string(),
            destination: accounts[instruction.accounts[2] as usize].to_string(),
            source: accounts[instruction.accounts[0] as usize].to_string(),
        },
        mint: token_info.mint.clone(),
        decimals: token_info.amount.decimals,
    };

    if transfer_data.mint.is_empty() {
        transfer_data.mint = "Unknown".to_string();
    }

    Ok(transfer_data)
}
