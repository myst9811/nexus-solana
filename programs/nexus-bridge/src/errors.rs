use anchor_lang::prelude::*;

/// Minimum amount for lock/unlock operations (1 token with 9 decimals = 1_000_000_000)
pub const MIN_BRIDGE_AMOUNT: u64 = 1_000_000_000;
/// Maximum amount per single lock/unlock operation (1 billion tokens with 9 decimals)
pub const MAX_BRIDGE_AMOUNT: u64 = 1_000_000_000_000_000_000;

#[error_code]
pub enum BridgeError {
    #[msg("Amount must be greater than 0")]
    InvalidAmount,

    #[msg("Amount is below minimum bridge threshold")]
    AmountBelowMinimum,

    #[msg("Amount exceeds maximum bridge limit")]
    AmountExceedsMaximum,

    #[msg("Invalid Ethereum address format")]
    InvalidEthAddress,

    #[msg("Transaction has already been processed")]
    TransactionAlreadyProcessed,

    #[msg("Unauthorized: caller is not the bridge authority")]
    Unauthorized,

    #[msg("Insufficient balance in bridge")]
    InsufficientBridgeBalance,

    #[msg("Token account mismatch")]
    TokenAccountMismatch,

    #[msg("Invalid transaction hash")]
    InvalidTxHash,
}
