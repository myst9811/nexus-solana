use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::{BridgeState, LockEvent};
use crate::errors::BridgeError;

#[derive(Accounts)]
#[instruction(amount: u64, eth_address: String)]
pub struct LockTokens<'info> {
    #[account(
        mut,
        seeds = [b"bridge"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, BridgeState>,
    
    #[account(
        init,
        payer = user,
        space = LockEvent::LEN,
        seeds = [b"lock_event", bridge_state.nonce.to_le_bytes().as_ref()],
        bump
    )]
    pub lock_event: Account<'info, LockEvent>,
    
    #[account(mut)]
    pub user: Signer<'info>,
    
    #[account(
        mut,
        constraint = user_token_account.owner == user.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = bridge_token_account.owner == bridge_state.key()
    )]
    pub bridge_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<LockTokens>,
    amount: u64,
    eth_address: String,
) -> Result<()> {
    // Validate inputs
    require!(amount > 0, BridgeError::InvalidAmount);
    require!(
        eth_address.len() == 42 && eth_address.starts_with("0x"),
        BridgeError::InvalidEthAddress
    );

    // Transfer tokens from user to bridge
    let cpi_accounts = Transfer {
        from: ctx.accounts.user_token_account.to_account_info(),
        to: ctx.accounts.bridge_token_account.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    // Update bridge state
    let bridge_state = &mut ctx.accounts.bridge_state;
    bridge_state.total_locked = bridge_state.total_locked
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    
    let event_nonce = bridge_state.nonce;
    bridge_state.nonce = bridge_state.nonce
        .checked_add(1)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Record lock event
    let lock_event = &mut ctx.accounts.lock_event;
    lock_event.user = ctx.accounts.user.key();
    lock_event.amount = amount;
    lock_event.eth_address = eth_address.clone();
    lock_event.timestamp = Clock::get()?.unix_timestamp;
    lock_event.processed = false;
    lock_event.nonce = event_nonce;

    msg!("Tokens locked: {} to Ethereum address: {}", amount, eth_address);
    
    Ok(())
}
