use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod errors;

pub use instructions::*;
pub use state::*;
pub use errors::*;

declare_id!("E8biA2oy2gbMRRWmvU66N9vAmb8qCFmNE2P5qGuTTY1f");

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
