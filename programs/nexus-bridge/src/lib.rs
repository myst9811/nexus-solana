use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod errors;

use instructions::*;
use state::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod nexus_bridge {
    use super::*;

    /// Initialize the bridge program
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    /// Lock SPL tokens to bridge to Ethereum
    pub fn lock_tokens(
        ctx: Context<LockTokens>,
        amount: u64,
        eth_address: String,
    ) -> Result<()> {
        instructions::lock_tokens::handler(ctx, amount, eth_address)
    }

    /// Unlock SPL tokens when bridging from Ethereum
    pub fn unlock_tokens(
        ctx: Context<UnlockTokens>,
        amount: u64,
        eth_tx_hash: String,
    ) -> Result<()> {
        instructions::unlock_tokens::handler(ctx, amount, eth_tx_hash)
    }
}
