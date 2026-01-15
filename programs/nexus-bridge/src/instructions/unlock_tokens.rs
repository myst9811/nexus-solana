use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::{BridgeState, ProcessedTransaction};
use crate::errors::{BridgeError, MIN_BRIDGE_AMOUNT, MAX_BRIDGE_AMOUNT};

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
        seeds = [b"processed_tx", &eth_tx_hash.as_bytes()[2..34]],
        bump
    )]
    pub processed_tx: Account<'info, ProcessedTransaction>,

    #[account(mut)]
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

/// Validates that a string contains only valid hexadecimal characters
fn is_valid_hex(s: &str) -> bool {
    s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Converts a hex string (without 0x prefix) to a 32-byte array
fn hex_to_bytes(hex: &str) -> Option<[u8; 32]> {
    if hex.len() != 64 {
        return None;
    }
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        bytes[i] = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(bytes)
}

pub fn handler_unlock_tokens(
    ctx: Context<UnlockTokens>,
    amount: u64,
    eth_tx_hash: String,
) -> Result<()> {
    // Validate amount
    require!(amount > 0, BridgeError::InvalidAmount);
    require!(amount >= MIN_BRIDGE_AMOUNT, BridgeError::AmountBelowMinimum);
    require!(amount <= MAX_BRIDGE_AMOUNT, BridgeError::AmountExceedsMaximum);

    // Validate Ethereum transaction hash format (0x + 64 hex chars)
    require!(
        eth_tx_hash.len() == 66
            && eth_tx_hash.starts_with("0x")
            && is_valid_hex(&eth_tx_hash[2..]),
        BridgeError::InvalidTxHash
    );

    // Check bridge has sufficient balance
    require!(
        ctx.accounts.bridge_token_account.amount >= amount,
        BridgeError::InsufficientBridgeBalance
    );

    // Transfer tokens from bridge to recipient
    let bridge_bump = ctx.accounts.bridge_state.bump;
    let seeds: &[&[u8]] = &[
        b"bridge",
        &[bridge_bump],
    ];
    let signer = &[seeds];

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
