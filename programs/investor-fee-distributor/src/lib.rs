pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use error::*;
pub use events::*;
pub use instructions::*;
pub use state::*;
pub use utils::*;

// Declare external programs for CPI
declare_program!(dynamic_amm);
declare_program!(dynamic_vault);

declare_id!("2UsVYuZY3pEWhWZceW7rH9gyDwLmzYXMuw3exduuAYmn");

#[program]
pub mod investor_fee_distributor {
    use super::*;

    /// Initialize fee distribution policy configuration
    pub fn initialize_policy(
        ctx: Context<InitializePolicy>,
        vault: Pubkey,
        investor_fee_share_bps: u16,
        daily_cap_lamports: Option<u64>,
        min_payout_lamports: u64,
        y0_total_streamed: u64,
        creator_quote_ata: Pubkey,
    ) -> Result<()> {
        instructions::initialize_policy::handler(
            ctx,
            vault,
            investor_fee_share_bps,
            daily_cap_lamports,
            min_payout_lamports,
            y0_total_streamed,
            creator_quote_ata,
        )
    }

    /// Initialize honorary DAMM v2 position for quote-only fee collection
    pub fn initialize_honorary_position(
        ctx: Context<InitializeHonoraryPosition>,
        vault: Pubkey,
    ) -> Result<()> {
        instructions::initialize_honorary_position::handler(ctx, vault)
    }

    /// Permissionless 24-hour distribution crank (manual fee transfer version)
    /// Use this when fees are manually transferred to treasury
    pub fn crank_distribution(
        ctx: Context<CrankDistribution>,
        total_pages: u16,
        investor_data: Vec<InvestorData>,
    ) -> Result<()> {
        instructions::crank_distribution::handler(ctx, total_pages, investor_data)
    }

    /// Permissionless 24-hour distribution crank (full CPI version)
    /// Use this when automatically claiming fees from lock escrow via CPI
    pub fn crank_distribution_full(
        ctx: Context<CrankDistributionFull>,
        total_pages: u16,
        investor_data: Vec<InvestorData>,
    ) -> Result<()> {
        instructions::crank_distribution_full::handler_full(ctx, total_pages, investor_data)
    }
}
