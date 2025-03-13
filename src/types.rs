use solana_sdk::{pubkey::Pubkey, signature::Signature};

pub struct Event {
    pub signature: Signature,
    pub event_type: EventType,
    pub timestamp: u64,
}

pub enum EventType {
    Swap(Swap),
    PoolCreation {
        mint: Pubkey,
        user: Pubkey,
        platform: SwapPlatform,
    },
    AssociatedTokenCreation {
        mint: Pubkey,
        user: Pubkey,
    },
}

#[derive(Debug, Clone)]
pub struct Swap {
    pub mint: Pubkey,
    pub user: Pubkey,
    pub is_buy: bool,
    pub sol_amount: u64,
    pub token_amount: u64,
    pub token_decimals: u8,
    pub platform: SwapPlatform,
    /// Including decimals
    pub token_reserve: f64,
    /// Including decimals
    pub sol_reserve: f64,
}

#[derive(Debug, Clone)]
pub enum SwapPlatform {
    PumpFun,
    Raydium,
    RaydiumCpmm,
    RaydiumClmm,
}
