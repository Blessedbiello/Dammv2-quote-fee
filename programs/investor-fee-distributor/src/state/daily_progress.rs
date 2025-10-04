use anchor_lang::prelude::*;

/// Tracks progress of fee distribution within a 24-hour window
#[account]
#[derive(Default)]
pub struct DailyProgress {
    /// Bump seed for PDA derivation
    pub bump: u8,
    /// Vault this progress tracks
    pub vault: Pubkey,
    /// Day identifier (unix_timestamp / 86400)
    pub day_id: u64,
    /// Window start timestamp (day_id * 86400)
    pub window_start: i64,
    /// Last crank timestamp
    pub last_crank_ts: i64,
    /// Total quote fees claimed from pool today
    pub total_quote_claimed_today: u64,
    /// Total distributed to investors today
    pub investor_distributed_today: u64,
    /// Total distributed to creator today
    pub creator_distributed_today: u64,
    /// Dust carried over from previous pages
    pub carry_over_lamports: u64,
    /// Current page number (0-indexed)
    pub current_page: u16,
    /// Total pages for this day
    pub total_pages: u16,
    /// Whether this day is finalized
    pub is_finalized: bool,
    /// Reserved for future upgrades
    pub reserved: [u8; 32],
}

impl DailyProgress {
    pub const INIT_SPACE: usize =
        1 +     // bump
        32 +    // vault
        8 +     // day_id
        8 +     // window_start
        8 +     // last_crank_ts
        8 +     // total_quote_claimed_today
        8 +     // investor_distributed_today
        8 +     // creator_distributed_today
        8 +     // carry_over_lamports
        2 +     // current_page
        2 +     // total_pages
        1 +     // is_finalized
        32;     // reserved

    /// Check if within the current 24h window
    pub fn is_within_window(&self, current_time: i64) -> bool {
        current_time >= self.window_start && current_time < self.window_start + 86400
    }

    /// Check if can crank
    pub fn can_crank(&self, current_time: i64) -> bool {
        self.is_within_window(current_time) && !self.is_finalized
    }

    /// Check if day is complete
    pub fn is_complete(&self) -> bool {
        self.current_page >= self.total_pages
    }

    /// Reset for new day
    pub fn reset_for_new_day(&mut self, day_id: u64, current_time: i64) {
        self.day_id = day_id;
        self.window_start = (day_id * 86400) as i64;
        self.last_crank_ts = current_time;
        self.total_quote_claimed_today = 0;
        self.investor_distributed_today = 0;
        self.creator_distributed_today = 0;
        self.carry_over_lamports = 0;
        self.current_page = 0;
        self.is_finalized = false;
    }
}
