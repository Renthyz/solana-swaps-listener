pub(super) use solana_sdk::pubkey::Pubkey;

pub mod programs {
    use super::Pubkey;

    pub const PUMPFUN: Pubkey = solana_sdk::pubkey!("6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P");
    pub const RAYDIUM: Pubkey = solana_sdk::pubkey!("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8");
    pub const RAYDIUM_CPMM: Pubkey =
        solana_sdk::pubkey!("CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");
    pub const RAYDIUM_CLMM: Pubkey =
        solana_sdk::pubkey!("CLMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C");
    pub const TOKEN: Pubkey = solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
    pub const TOKEN_2022: Pubkey =
        solana_sdk::pubkey!("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
}

pub const SOLANA_PUBKEY: Pubkey =
    solana_sdk::pubkey!("So11111111111111111111111111111111111111112");
