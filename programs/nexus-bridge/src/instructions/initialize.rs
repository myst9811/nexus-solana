use anchor_lang::prelude::*;
use crate::state::BridgeState;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = BridgeState::LEN,
        seeds = [b"bridge"],
        bump
    )]
    pub bridge_state: Account<'info, BridgeState>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let bridge_state = &mut ctx.accounts.bridge_state;
    
    bridge_state.authority = ctx.accounts.authority.key();
    bridge_state.total_locked = 0;
    bridge_state.total_unlocked = 0;
    bridge_state.nonce = 0;
    bridge_state.bump = ctx.bumps.bridge_state;
    
    msg!("Bridge initialized with authority: {}", bridge_state.authority);
    
    Ok(())
}
