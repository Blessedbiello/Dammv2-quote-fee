use anchor_lang::prelude::*;

#[event]
pub struct HonoraryPositionInitialized {
    pub vault: Pubkey,
    pub pool: Pubkey,
    pub position: Pubkey,
    pub quote_mint: Pubkey,
    pub base_mint: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct QuoteFeesClaimed {
    pub day_id: u64,
    pub amount_claimed: u64,
    pub position: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct InvestorPayoutPage {
    pub day_id: u64,
    pub page: u16,
    pub investors_paid: u16,
    pub total_distributed: u64,
    pub dust_carried: u64,
    pub timestamp: i64,
}

#[event]
pub struct CreatorPayoutDayClosed {
    pub day_id: u64,
    pub creator_amount: u64,
    pub total_investors_paid: u64,
    pub total_pages: u16,
    pub timestamp: i64,
}

#[event]
pub struct PolicyConfigCreated {
    pub vault: Pubkey,
    pub authority: Pubkey,
    pub investor_fee_share_bps: u16,
    pub y0_total_streamed: u64,
    pub timestamp: i64,
}

#[event]
pub struct DailyProgressReset {
    pub vault: Pubkey,
    pub old_day_id: u64,
    pub new_day_id: u64,
    pub timestamp: i64,
}
