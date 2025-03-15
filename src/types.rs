use solana_sdk::{pubkey::Pubkey, signature::Signature};
use strum::{Display, EnumIter};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Event {
    pub signature: Signature,
    pub event_type: EventType,
    pub user: Pubkey,
    pub timestamp: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    Swap(Swap),
    PoolCreation {
        mint: Pubkey,
        platform: SwapPlatform,
    },
    AssociatedAccountCreation {
        mint: Pubkey,
        account: Pubkey,
        idempotent: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Swap {
    pub token_in_amount: u64,
    pub token_in_decimals: u8,
    pub token_in_mint: Pubkey,

    pub token_out_amount: u64,
    pub token_out_decimals: u8,
    pub token_out_mint: Pubkey,

    pub platform: SwapPlatform,
    /// Including decimals
    pub token_in_reserve: u64,
    /// Including decimals
    pub token_out_reserve: u64,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, EnumIter, Display)]
pub enum SwapPlatform {
    PumpFun,
    Raydium,
    RaydiumCpmm,
    RaydiumClmm,
}
