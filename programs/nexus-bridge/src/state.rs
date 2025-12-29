use anchor_lang::prelude::*;

/// Main bridge state account
#[account]
#[derive(Default)]
pub struct BridgeState {
    /// Authority that can perform admin operations
    pub authority: Pubkey,
    /// Total amount of tokens locked
    pub total_locked: u64,
    /// Total amount of tokens unlocked
    pub total_unlocked: u64,
    /// Nonce for generating unique PDAs
    pub nonce: u64,
    /// Bump seed for PDA derivation
    pub bump: u8,
}

impl BridgeState {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        8 +  // total_locked
        8 +  // total_unlocked
        8 +  // nonce
        1;   // bump
}

/// Lock event record
#[account]
#[derive(Default)]
pub struct LockEvent {
    /// User who locked the tokens
    pub user: Pubkey,
    /// Amount locked
    pub amount: u64,
    /// Ethereum address to receive wrapped tokens
    pub eth_address: String,
    /// Timestamp of lock
    pub timestamp: i64,
    /// Whether this event has been processed
    pub processed: bool,
    /// Nonce for uniqueness
    pub nonce: u64,
}

impl LockEvent {
    pub const MAX_ETH_ADDRESS_LEN: usize = 42; // 0x + 40 hex chars
    
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        8 +  // amount
        4 + Self::MAX_ETH_ADDRESS_LEN + // eth_address (string with length prefix)
        8 +  // timestamp
        1 +  // processed
        8;   // nonce
}

/// Processed transaction tracker (prevents replay attacks)
#[account]
#[derive(Default)]
pub struct ProcessedTransaction {
    /// Ethereum transaction hash
    pub eth_tx_hash: String,
    /// When it was processed
    pub processed_at: i64,
    /// Amount that was unlocked
    pub amount: u64,
}

impl ProcessedTransaction {
    pub const MAX_TX_HASH_LEN: usize = 66; // 0x + 64 hex chars
    
    pub const LEN: usize = 8 + // discriminator
        4 + Self::MAX_TX_HASH_LEN + // eth_tx_hash
        8 +  // processed_at
        8;   // amount
}
