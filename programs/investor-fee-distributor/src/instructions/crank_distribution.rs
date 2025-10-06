use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::{constants::*, error::ErrorCode, events::*, state::*, utils::*};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InvestorData {
    /// Index in the investor list (for ordering)
    pub index: u32,
    /// Investor's quote token ATA
    pub quote_ata: Pubkey,
}

#[derive(Accounts)]
#[instruction(total_pages: u16)]
pub struct CrankDistribution<'info> {
    /// Cranker (permissionless - anyone can call)
    #[account(mut)]
    pub cranker: Signer<'info>,

    /// Daily progress tracking account
    #[account(
        init_if_needed,
        payer = cranker,
        space = 8 + DailyProgress::INIT_SPACE,
        seeds = [DAILY_PROGRESS_SEED, investor_fee_position_owner.vault.as_ref()],
        bump
    )]
    pub daily_progress: Account<'info, DailyProgress>,

    /// Policy configuration
    #[account(
        seeds = [POLICY_CONFIG_SEED, investor_fee_position_owner.vault.as_ref()],
        bump = policy_config.bump,
    )]
    pub policy_config: Account<'info, PolicyConfig>,

    /// Honorary position owner PDA
    #[account(
        seeds = [INVESTOR_FEE_POS_OWNER_SEED, investor_fee_position_owner.vault.as_ref()],
        bump = investor_fee_position_owner.bump,
    )]
    pub investor_fee_position_owner: Account<'info, InvestorFeePositionOwner>,

    /// Treasury quote token account (owned by investor_fee_position_owner)
    #[account(
        mut,
        constraint = treasury_quote_ata.mint == investor_fee_position_owner.quote_mint @ ErrorCode::InvalidTokenMint,
        constraint = treasury_quote_ata.owner == investor_fee_position_owner.key() @ ErrorCode::InvalidPosition,
    )]
    pub treasury_quote_ata: Account<'info, TokenAccount>,

    /// Treasury base token account (should be empty - for validation)
    #[account(
        constraint = treasury_base_ata.mint == investor_fee_position_owner.base_mint @ ErrorCode::InvalidTokenMint,
        constraint = treasury_base_ata.owner == investor_fee_position_owner.key() @ ErrorCode::InvalidPosition,
    )]
    pub treasury_base_ata: Account<'info, TokenAccount>,

    /// Creator's quote token ATA (for remainder)
    #[account(
        mut,
        constraint = creator_quote_ata.key() == policy_config.creator_quote_ata @ ErrorCode::InvalidPolicy,
        constraint = creator_quote_ata.mint == investor_fee_position_owner.quote_mint @ ErrorCode::InvalidTokenMint,
    )]
    pub creator_quote_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    // Remaining accounts:
    // 1. Streamflow stream accounts (read-only) - for reading locked amounts
    // 2. Investor quote ATAs (writable) - for transferring fees
    // Pattern: [stream_0, stream_1, ..., stream_n, ata_0, ata_1, ..., ata_n]
}

pub fn handler<'info>(
    ctx: Context<'_, '_, '_, 'info, CrankDistribution<'info>>,
    total_pages: u16,
    investor_data: Vec<InvestorData>,
) -> Result<()> {
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;
    let day_id = (current_time / SECONDS_PER_DAY) as u64;

    let progress = &mut ctx.accounts.daily_progress;
    let policy = &ctx.accounts.policy_config;
    let position_owner = &ctx.accounts.investor_fee_position_owner;

    // Validate inputs
    require!(total_pages > 0, ErrorCode::InvalidTotalPages);
    require!(!investor_data.is_empty(), ErrorCode::InvalidInvestorPage);

    // ===== STEP 1: 24H GATE & DAY INITIALIZATION =====

    if progress.day_id == 0 {
        // First time initialization
        progress.bump = ctx.bumps.daily_progress;
        progress.vault = position_owner.vault;
        progress.reset_for_new_day(day_id, current_time);
        progress.total_pages = total_pages;
    } else if progress.day_id != day_id {
        // New day - check 24h gate
        require!(
            current_time >= progress.window_start + SECONDS_PER_DAY,
            ErrorCode::TooEarlyForNextDay
        );

        // Emit event for day transition
        emit!(DailyProgressReset {
            vault: position_owner.vault,
            old_day_id: progress.day_id,
            new_day_id: day_id,
            timestamp: current_time,
        });

        // Reset for new day
        progress.reset_for_new_day(day_id, current_time);
        progress.total_pages = total_pages;
    }

    // Check if already finalized
    require!(!progress.is_finalized, ErrorCode::DayAlreadyFinalized);

    // Validate within window
    require!(
        progress.is_within_window(current_time),
        ErrorCode::OutsideWindow
    );

    // ===== STEP 2: CLAIM FEES (First page only) =====

    if progress.current_page == 0 {
        // MANUAL VERSION: Fees must be manually transferred to treasury_quote_ata
        // before calling this instruction. This version does not perform CPI to claim fees.
        // For automatic fee claiming via CPI, use crank_distribution_full instead.

        msg!("Manual crank version - using pre-transferred treasury balance");

        let quote_fees = ctx.accounts.treasury_quote_ata.amount;

        progress.total_quote_claimed_today = quote_fees;
        progress.carry_over_lamports = 0;

        emit!(QuoteFeesClaimed {
            day_id,
            amount_claimed: quote_fees,
            position: position_owner.lock_escrow,
            timestamp: current_time,
        });
    }

    // ===== STEP 3: VALIDATE QUOTE-ONLY =====

    let base_balance = ctx.accounts.treasury_base_ata.amount;
    require!(base_balance == 0, ErrorCode::BaseFeesDetected);

    // ===== STEP 4: CALCULATE LOCKED AMOUNTS FROM STREAMFLOW =====

    let num_investors = investor_data.len();

    // Remaining accounts split: first half are stream accounts, second half are ATAs
    require!(
        ctx.remaining_accounts.len() == num_investors * 2,
        ErrorCode::InvalidInvestorPage
    );

    // Calculate total locked across all investors
    let locked_total = calculate_total_locked(&ctx.remaining_accounts[0..num_investors], current_time)?;

    // If nothing is locked, all fees go to creator
    if locked_total == 0 {
        msg!("No tokens locked - all fees will go to creator");

        // Skip to finalization
        progress.current_page = progress.total_pages;
        progress.investor_distributed_today = 0;

        // Transfer all fees to creator (will happen in finalization step below)
    } else {
        // ===== STEP 5: CALCULATE INVESTOR SHARE =====

        let f_locked_bps = calculate_f_locked_bps(locked_total, policy.y0_total_streamed)?;
        let eligible_investor_share_bps = f_locked_bps.min(policy.investor_fee_share_bps as u64);

        let total_available = progress.total_quote_claimed_today
            .checked_add(progress.carry_over_lamports)
            .ok_or(ErrorCode::ArithmeticOverflow)?;

        let mut investor_fee_quote = apply_bps(total_available, eligible_investor_share_bps as u16)?;

        // Apply daily cap if configured
        if let Some(cap) = policy.daily_cap_lamports {
            let already_paid = progress.investor_distributed_today;
            let remaining_cap = cap.saturating_sub(already_paid);
            investor_fee_quote = investor_fee_quote.min(remaining_cap);

            if already_paid >= cap {
                return Err(ErrorCode::DailyCapReached.into());
            }
        }

        // ===== STEP 6: DISTRIBUTE TO INVESTORS PRO-RATA =====

        let mut total_distributed_this_page = 0u64;
        let mut dust_accumulator = progress.carry_over_lamports;

        for (i, _investor) in investor_data.iter().enumerate() {
            // Get references to stream and investor ATA upfront
            let stream_account = &ctx.remaining_accounts[i];
            let investor_ata_account = &ctx.remaining_accounts[num_investors + i];

            // Parse locked amount for this investor
            let stream = parse_streamflow_stream(stream_account)?;
            let locked_i = stream.calculate_locked_at_timestamp(current_time)?;

            // Calculate pro-rata payout
            let payout = calculate_pro_rata_share(
                investor_fee_quote,
                locked_i,
                locked_total,
            )?;

            if payout >= policy.min_payout_lamports {
                // Transfer quote tokens to investor
                let seeds = &[
                    INVESTOR_FEE_POS_OWNER_SEED,
                    position_owner.vault.as_ref(),
                    &[position_owner.bump],
                ];
                let signer_seeds = &[&seeds[..]];

                token::transfer(
                    CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.treasury_quote_ata.to_account_info(),
                            to: investor_ata_account.clone(),
                            authority: position_owner.to_account_info(),
                        },
                        signer_seeds,
                    ),
                    payout,
                )?;

                total_distributed_this_page = total_distributed_this_page
                    .checked_add(payout)
                    .ok_or(ErrorCode::ArithmeticOverflow)?;
            } else {
                // Below dust threshold - carry forward
                dust_accumulator = dust_accumulator
                    .checked_add(payout)
                    .ok_or(ErrorCode::ArithmeticOverflow)?;
            }
        }

        // ===== STEP 7: UPDATE PROGRESS =====

        progress.investor_distributed_today = progress.investor_distributed_today
            .checked_add(total_distributed_this_page)
            .ok_or(ErrorCode::ArithmeticOverflow)?;

        progress.carry_over_lamports = dust_accumulator;
        progress.current_page = progress.current_page.saturating_add(1);
        progress.last_crank_ts = current_time;

        emit!(InvestorPayoutPage {
            day_id,
            page: progress.current_page,
            investors_paid: investor_data.len() as u16,
            total_distributed: total_distributed_this_page,
            dust_carried: dust_accumulator,
            timestamp: current_time,
        });
    }

    // ===== STEP 8: FINALIZE DAY (if last page) =====

    if progress.current_page >= progress.total_pages {
        let creator_remainder = progress.total_quote_claimed_today
            .saturating_sub(progress.investor_distributed_today);

        if creator_remainder > 0 {
            // Transfer remainder to creator
            let seeds = &[
                INVESTOR_FEE_POS_OWNER_SEED,
                position_owner.vault.as_ref(),
                &[position_owner.bump],
            ];
            let signer_seeds = &[&seeds[..]];

            let cpi_accounts = Transfer {
                from: ctx.accounts.treasury_quote_ata.to_account_info(),
                to: ctx.accounts.creator_quote_ata.to_account_info(),
                authority: position_owner.to_account_info(),
            };

            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );

            token::transfer(cpi_ctx, creator_remainder)?;

            progress.creator_distributed_today = creator_remainder;
        }

        progress.is_finalized = true;

        emit!(CreatorPayoutDayClosed {
            day_id,
            creator_amount: creator_remainder,
            total_investors_paid: progress.investor_distributed_today,
            total_pages: progress.total_pages,
            timestamp: current_time,
        });
    }

    Ok(())
}
