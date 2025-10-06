use anchor_lang::prelude::*;

/// PDA that owns the honorary DAMM v2 position for fee collection
#[account]
pub struct InvestorFeePositionOwner {
    /// Bump seed for PDA derivation
    pub bump: u8,
    /// Vault this position serves
    pub vault: Pubkey,
    /// DAMM v2 pool address
    pub pool: Pubkey,
    /// Lock escrow (honorary position in DAMM v2)
    pub lock_escrow: Pubkey,
    /// LP mint of the pool
    pub lp_mint: Pubkey,
    /// Quote mint (Token B in DAMM v2)
    pub quote_mint: Pubkey,
    /// Base mint (Token A in DAMM v2)
    pub base_mint: Pubkey,
    /// Timestamp when position was created
    pub created_at: i64,
    /// Last fee claim timestamp
    pub last_fee_claim: i64,
    /// Lifetime total quote fees claimed
    pub total_fees_claimed: u64,
    /// Reserved for future upgrades
    pub reserved: [u8; 64],
}

impl InvestorFeePositionOwner {
    pub const INIT_SPACE: usize =
        1 +     // bump
        32 +    // vault
        32 +    // pool
        32 +    // lock_escrow
        32 +    // lp_mint
        32 +    // quote_mint
        32 +    // base_mint
        8 +     // created_at
        8 +     // last_fee_claim
        8 +     // total_fees_claimed
        64;     // reserved
}
