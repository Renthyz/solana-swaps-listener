use solana_sdk::{pubkey::Pubkey, signature::Signature};

#[derive(Debug, Clone)]
pub struct Event {
    pub signature: Signature,
    pub event_type: EventType,
    pub user: Pubkey,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum EventType {
    Swap(Swap),
    PoolCreation {
        mint: Pubkey,
        platform: SwapPlatform,
    },
    AssociatedTokenCreation {
        mint: Pubkey,
    },
}

#[derive(Debug, Clone)]
pub struct Swap {
    pub token_in_amount: u64,
    pub token_in_decimals: u8,
    pub token_in_mint: Pubkey,

    pub token_out_amount: u64,
    pub token_out_decimals: u8,
    pub token_out_mint: Pubkey,

    pub platform: SwapPlatform,
    /// Including decimals
    pub token_in_reserve: f64,
    /// Including decimals
    pub token_out_reserve: f64,
}

#[derive(Debug, Clone)]
pub enum SwapPlatform {
    PumpFun,
    Raydium,
    RaydiumCpmm,
    RaydiumClmm,
}
