use anchor_lang::prelude::*;
use crate::{constants::*, error::ErrorCode, events::*, state::*};

#[derive(Accounts)]
#[instruction(vault: Pubkey)]
pub struct InitializePolicy<'info> {
    #[account(
        init,
        payer = payer,
        space = 8 + PolicyConfig::INIT_SPACE,
        seeds = [POLICY_CONFIG_SEED, vault.as_ref()],
        bump
    )]
    pub policy_config: Account<'info, PolicyConfig>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<InitializePolicy>,
    vault: Pubkey,
    investor_fee_share_bps: u16,
    daily_cap_lamports: Option<u64>,
    min_payout_lamports: u64,
    y0_total_streamed: u64,
    creator_quote_ata: Pubkey,
) -> Result<()> {
    // Validate inputs
    require!(
        investor_fee_share_bps <= MAX_BPS,
        ErrorCode::InvalidFeeShareBps
    );
    require!(y0_total_streamed > 0, ErrorCode::InvalidY0Amount);

    let policy = &mut ctx.accounts.policy_config;
    let clock = Clock::get()?;

    policy.bump = ctx.bumps.policy_config;
    policy.authority = ctx.accounts.authority.key();
    policy.vault = vault;
    policy.investor_fee_share_bps = investor_fee_share_bps;
    policy.daily_cap_lamports = daily_cap_lamports;
    policy.min_payout_lamports = min_payout_lamports;
    policy.y0_total_streamed = y0_total_streamed;
    policy.creator_quote_ata = creator_quote_ata;

    emit!(PolicyConfigCreated {
        vault,
        authority: ctx.accounts.authority.key(),
        investor_fee_share_bps,
        y0_total_streamed,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
