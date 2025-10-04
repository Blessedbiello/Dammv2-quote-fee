use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::{constants::*, error::ErrorCode, events::*, state::*, dynamic_amm};

#[derive(Accounts)]
#[instruction(vault: Pubkey)]
pub struct InitializeHonoraryPosition<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + InvestorFeePositionOwner::INIT_SPACE,
        seeds = [INVESTOR_FEE_POS_OWNER_SEED, vault.as_ref()],
        bump
    )]
    pub investor_fee_position_owner: Account<'info, InvestorFeePositionOwner>,

    #[account(
        seeds = [POLICY_CONFIG_SEED, vault.as_ref()],
        bump = policy_config.bump,
    )]
    pub policy_config: Account<'info, PolicyConfig>,

    /// CHECK: DAMM v2 pool account - validated by cp-amm program
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,

    /// LP mint of the pool
    pub lp_mint: Account<'info, Mint>,

    /// CHECK: Lock escrow to be created by cp-amm
    #[account(mut)]
    pub lock_escrow: UncheckedAccount<'info>,

    /// Quote mint (Token B in pool)
    pub quote_mint: Account<'info, Mint>,

    /// Base mint (Token A in pool)
    pub base_mint: Account<'info, Mint>,

    /// Treasury ATA for quote token (owned by investor_fee_position_owner PDA)
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = quote_mint,
        associated_token::authority = investor_fee_position_owner
    )]
    pub treasury_quote_ata: Account<'info, TokenAccount>,

    /// Treasury ATA for base token (should remain empty - for validation)
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = base_mint,
        associated_token::authority = investor_fee_position_owner
    )]
    pub treasury_base_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    /// CHECK: Dynamic AMM program
    #[account(address = dynamic_amm::ID)]
    pub dynamic_amm_program: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializeHonoraryPosition>,
    vault: Pubkey,
) -> Result<()> {
    let clock = Clock::get()?;

    // Step 1: Validate pool configuration for quote-only fees
    // NOTE: This requires parsing the pool's config account to check collectFeeMode
    // For production, implement config validation here
    // For now, we trust the pool is configured correctly

    msg!("Creating honorary lock escrow for quote-only fee collection");

    // Step 2: Create lock escrow via CPI to dynamic_amm
    let seeds = &[
        INVESTOR_FEE_POS_OWNER_SEED,
        vault.as_ref(),
        &[ctx.bumps.investor_fee_position_owner],
    ];
    let signer_seeds = &[&seeds[..]];

    let cpi_accounts = dynamic_amm::cpi::accounts::CreateLockEscrow {
        pool: ctx.accounts.pool.to_account_info(),
        lock_escrow: ctx.accounts.lock_escrow.to_account_info(),
        owner: ctx.accounts.investor_fee_position_owner.to_account_info(),
        lp_mint: ctx.accounts.lp_mint.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.dynamic_amm_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );

    dynamic_amm::cpi::create_lock_escrow(cpi_ctx)?;

    msg!("Lock escrow created successfully");

    // Step 3: Initialize state
    let owner = &mut ctx.accounts.investor_fee_position_owner;
    owner.bump = ctx.bumps.investor_fee_position_owner;
    owner.vault = vault;
    owner.pool = ctx.accounts.pool.key();
    owner.lock_escrow = ctx.accounts.lock_escrow.key();
    owner.lp_mint = ctx.accounts.lp_mint.key();
    owner.quote_mint = ctx.accounts.quote_mint.key();
    owner.base_mint = ctx.accounts.base_mint.key();
    owner.created_at = clock.unix_timestamp;
    owner.last_fee_claim = clock.unix_timestamp;
    owner.total_fees_claimed = 0;

    emit!(HonoraryPositionInitialized {
        vault,
        pool: ctx.accounts.pool.key(),
        position: ctx.accounts.lock_escrow.key(),
        quote_mint: ctx.accounts.quote_mint.key(),
        base_mint: ctx.accounts.base_mint.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("Honorary position initialized - ready to accrue quote-only fees");

    Ok(())
}
