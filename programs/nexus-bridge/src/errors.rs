use anchor_lang::prelude::*;

#[error_code]
pub enum BridgeError {
    #[msg("Amount must be greater than 0")]
    InvalidAmount,
    
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
