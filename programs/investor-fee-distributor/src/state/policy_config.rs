use anchor_lang::prelude::*;

/// Configuration for fee distribution policy
#[account]
pub struct PolicyConfig {
    /// Bump seed for PDA derivation
    pub bump: u8,
    /// Authority that can update policy (Star team)
    pub authority: Pubkey,
    /// Vault this policy serves
    pub vault: Pubkey,
    /// Maximum investor fee share in basis points (e.g., 7000 = 70%)
    pub investor_fee_share_bps: u16,
    /// Optional daily cap in lamports (quote token)
    pub daily_cap_lamports: Option<u64>,
    /// Minimum payout threshold - amounts below this are carried forward
    pub min_payout_lamports: u64,
    /// Total investor allocation minted at TGE (Y0)
    pub y0_total_streamed: u64,
    /// Creator's quote token ATA for receiving remainder
    pub creator_quote_ata: Pubkey,
    /// Reserved for future upgrades
    pub reserved: [u8; 64],
}

impl PolicyConfig {
    pub const INIT_SPACE: usize =
        1 +     // bump
        32 +    // authority
        32 +    // vault
        2 +     // investor_fee_share_bps
        1 + 8 + // Option<u64> for daily_cap_lamports
        8 +     // min_payout_lamports
        8 +     // y0_total_streamed
        32 +    // creator_quote_ata
        64;     // reserved
}
