use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::{BridgeState, ProcessedTransaction};
use crate::errors::BridgeError;

#[derive(Accounts)]
#[instruction(amount: u64, eth_tx_hash: String)]
pub struct UnlockTokens<'info> {
    #[account(
        mut,
        seeds = [b"bridge"],
        bump = bridge_state.bump,
        has_one = authority @ BridgeError::Unauthorized
    )]
    pub bridge_state: Account<'info, BridgeState>,

    #[account(
        init,
        payer = authority,
        space = ProcessedTransaction::LEN,
        seeds = [b"processed_tx", eth_tx_hash.as_bytes()],
        bump
    )]
    pub processed_tx: Account<'info, ProcessedTransaction>,
    
    pub authority: Signer<'info>,

    /// CHECK: This is the user receiving unlocked tokens
    pub recipient: AccountInfo<'info>,

    /// Token mint for the SPL token being bridged
    pub token_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = recipient_token_account.owner == recipient.key(),
        constraint = recipient_token_account.mint == token_mint.key()
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = bridge_state
    )]
    pub bridge_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<UnlockTokens>,
    amount: u64,
    eth_tx_hash: String,
) -> Result<()> {
    // Validate inputs
    require!(amount > 0, BridgeError::InvalidAmount);
    require!(
        eth_tx_hash.len() == 66 && eth_tx_hash.starts_with("0x"),
        BridgeError::InvalidTxHash
    );

    // Check bridge has sufficient balance
    require!(
        ctx.accounts.bridge_token_account.amount >= amount,
        BridgeError::InsufficientBridgeBalance
    );

    // Transfer tokens from bridge to recipient
    let seeds = &[
        b"bridge",
        &[ctx.accounts.bridge_state.bump],
    ];
    let signer = &[&seeds[..]];

    let cpi_accounts = Transfer {
        from: ctx.accounts.bridge_token_account.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: ctx.accounts.bridge_state.to_account_info(),
    };
    
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::transfer(cpi_ctx, amount)?;

    // Update bridge state
    let bridge_state = &mut ctx.accounts.bridge_state;
    bridge_state.total_unlocked = bridge_state.total_unlocked
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Record processed transaction
    let processed_tx = &mut ctx.accounts.processed_tx;
    processed_tx.eth_tx_hash = eth_tx_hash.clone();
    processed_tx.processed_at = Clock::get()?.unix_timestamp;
    processed_tx.amount = amount;

    msg!("Tokens unlocked: {} from Ethereum tx: {}", amount, eth_tx_hash);
    
    Ok(())
}
