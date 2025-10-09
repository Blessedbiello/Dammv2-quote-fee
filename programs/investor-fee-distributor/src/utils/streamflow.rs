use anchor_lang::prelude::*;
use crate::error::ErrorCode;

/// Streamflow stream account structure (simplified)
/// Full structure available at: https://github.com/streamflow-finance/js-sdk
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct StreamflowStream {
    /// Magic bytes to identify account type
    pub magic: u64,
    /// Version
    pub version: u64,
    /// Timestamp when stream was created
    pub created_at: u64,
    /// Amount of tokens withdrawn
    pub withdrawn_amount: u64,
    /// Timestamp when cancellation occurred (0 if not cancelled)
    pub canceled_at: u64,
    /// Timestamp when stream ends
    pub end_time: u64,
    /// Last withdrawn timestamp
    pub last_withdrawn_at: u64,
    /// Sender address
    pub sender: Pubkey,
    /// Sender tokens address
    pub sender_tokens: Pubkey,
    /// Recipient address
    pub recipient: Pubkey,
    /// Recipient tokens address
    pub recipient_tokens: Pubkey,
    /// Token mint
    pub mint: Pubkey,
    /// Escrow tokens address (where funds are held)
    pub escrow_tokens: Pubkey,
    /// Start time
    pub start_time: u64,
    /// Total deposited amount
    pub deposited_amount: u64,
    /// Period (vesting interval in seconds)
    pub period: u64,
    /// Amount per period
    pub amount_per_period: u64,
    /// Cliff period (seconds before vesting starts)
    pub cliff: u64,
    /// Cliff amount (released at cliff)
    pub cliff_amount: u64,
    /// Whether cancelable by sender
    pub cancelable_by_sender: bool,
    /// Whether cancelable by recipient
    pub cancelable_by_recipient: bool,
    /// Whether automatic withdrawal enabled
    pub automatic_withdrawal: bool,
    /// Whether transferable by sender
    pub transferable_by_sender: bool,
    /// Whether transferable by recipient
    pub transferable_by_recipient: bool,
    /// Whether stream can be topped up
    pub can_topup: bool,
    /// Stream name
    pub stream_name: [u8; 64],
    /// Whether pausable
    pub can_pause: bool,
    /// Pause cumulative time
    pub pause_cumulative: u64,
    /// Last paused at
    pub last_rate_change_time: u64,
    /// Funds unlocked at last rate change
    pub funds_unlocked_at_last_rate_change: u64,
}

impl StreamflowStream {
    /// Streamflow magic number identifier
    pub const MAGIC: u64 = 0x1a23f45e67b89c0d;

    /// Calculate the amount that is still locked at a given timestamp
    pub fn calculate_locked_at_timestamp(&self, current_time: i64) -> Result<u64> {
        let current_time = current_time as u64;

        // If stream is cancelled, everything is unlocked
        if self.canceled_at > 0 {
            return Ok(0);
        }

        // If before start time, everything is locked
        if current_time < self.start_time {
            return Ok(
                self.deposited_amount
                    .checked_sub(self.withdrawn_amount)
                    .ok_or(ErrorCode::ArithmeticUnderflow)?
            );
        }

        // If after end time, nothing is locked
        if current_time >= self.end_time {
            return Ok(0);
        }

        // Calculate vested amount at current time
        let time_elapsed = current_time
            .checked_sub(self.start_time)
            .ok_or(ErrorCode::ArithmeticUnderflow)?;

        // Handle cliff period
        let vested = if time_elapsed < self.cliff {
            // Before cliff, nothing vested
            0u64
        } else {
            // After cliff, cliff amount + linear vesting
            let time_after_cliff = time_elapsed
                .checked_sub(self.cliff)
                .ok_or(ErrorCode::ArithmeticUnderflow)?;

            let periods_elapsed = time_after_cliff
                .checked_div(self.period)
                .unwrap_or(0);

            let linear_vested = periods_elapsed
                .checked_mul(self.amount_per_period)
                .ok_or(ErrorCode::ArithmeticOverflow)?;

            self.cliff_amount
                .checked_add(linear_vested)
                .ok_or(ErrorCode::ArithmeticOverflow)?
                .min(self.deposited_amount)
        };

        // Locked = Deposited - Vested
        let locked = self.deposited_amount
            .checked_sub(vested)
            .ok_or(ErrorCode::ArithmeticUnderflow)?;

        Ok(locked)
    }

    /// Validate that this is a Streamflow account
    pub fn validate(&self) -> Result<()> {
        require!(
            self.magic == Self::MAGIC,
            ErrorCode::StreamflowAccountMismatch
        );
        Ok(())
    }
}

/// Parse a Streamflow stream account from account info
pub fn parse_streamflow_stream(account_info: &AccountInfo) -> Result<StreamflowStream> {
    // SECURITY: Validate account owner to prevent fake stream accounts
    // The account must be owned by the Streamflow program
    let streamflow_program_id = "strmRqUCoQUgGUan5YhzUZa6KqdzwX5L6FpUxfmKg5m"
        .parse::<Pubkey>()
        .unwrap();

    require!(
        account_info.owner == &streamflow_program_id,
        ErrorCode::StreamflowAccountMismatch
    );

    let data = account_info.try_borrow_data()?;

    // Minimum size check (basic validation)
    require!(
        data.len() >= 8, // At least magic number
        ErrorCode::StreamflowAccountMismatch
    );

    // Deserialize the stream
    let mut data_slice: &[u8] = &data;
    let stream = StreamflowStream::deserialize(&mut data_slice)
        .map_err(|_| ErrorCode::StreamflowAccountMismatch)?;

    // Validate it's a Streamflow account
    stream.validate()?;

    Ok(stream)
}

/// Calculate total locked amount across multiple streams at a given timestamp
pub fn calculate_total_locked(
    stream_accounts: &[AccountInfo],
    current_time: i64,
) -> Result<u64> {
    let mut total_locked = 0u64;

    for account_info in stream_accounts.iter() {
        let stream = parse_streamflow_stream(account_info)?;
        let locked = stream.calculate_locked_at_timestamp(current_time)?;

        total_locked = total_locked
            .checked_add(locked)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
    }

    Ok(total_locked)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_stream(
        start_time: u64,
        end_time: u64,
        deposited: u64,
        withdrawn: u64,
        period: u64,
        amount_per_period: u64,
        cliff: u64,
        cliff_amount: u64,
    ) -> StreamflowStream {
        StreamflowStream {
            magic: StreamflowStream::MAGIC,
            version: 1,
            created_at: start_time,
            withdrawn_amount: withdrawn,
            canceled_at: 0,
            end_time,
            last_withdrawn_at: start_time,
            sender: Pubkey::default(),
            sender_tokens: Pubkey::default(),
            recipient: Pubkey::default(),
            recipient_tokens: Pubkey::default(),
            mint: Pubkey::default(),
            escrow_tokens: Pubkey::default(),
            start_time,
            deposited_amount: deposited,
            period,
            amount_per_period,
            cliff,
            cliff_amount,
            cancelable_by_sender: false,
            cancelable_by_recipient: false,
            automatic_withdrawal: false,
            transferable_by_sender: false,
            transferable_by_recipient: false,
            can_topup: false,
            stream_name: [0u8; 64],
            can_pause: false,
            pause_cumulative: 0,
            last_rate_change_time: 0,
            funds_unlocked_at_last_rate_change: 0,
        }
    }

    #[test]
    fn test_before_start_time() {
        let stream = create_test_stream(
            1000, // start
            2000, // end
            1000, // deposited
            0,    // withdrawn
            100,  // period
            100,  // amount per period
            0,    // cliff
            0,    // cliff amount
        );

        // Before start: everything locked
        let locked = stream.calculate_locked_at_timestamp(500).unwrap();
        assert_eq!(locked, 1000);
    }

    #[test]
    fn test_after_end_time() {
        let stream = create_test_stream(
            1000, 2000, 1000, 0, 100, 100, 0, 0
        );

        // After end: nothing locked
        let locked = stream.calculate_locked_at_timestamp(2500).unwrap();
        assert_eq!(locked, 0);
    }

    #[test]
    fn test_linear_vesting() {
        // 1000 tokens over 1000 seconds = 1 per second
        let stream = create_test_stream(
            1000,  // start
            2000,  // end (1000 seconds duration)
            1000,  // deposited
            0,     // withdrawn
            1,     // period (1 second)
            1,     // amount per period
            0,     // no cliff
            0,
        );

        // At halfway point (1500): 500 vested, 500 locked
        let locked = stream.calculate_locked_at_timestamp(1500).unwrap();
        assert_eq!(locked, 500);

        // At 75% point (1750): 750 vested, 250 locked
        let locked = stream.calculate_locked_at_timestamp(1750).unwrap();
        assert_eq!(locked, 250);
    }

    #[test]
    fn test_cliff_vesting() {
        // 1000 tokens with 500 second cliff, 500 cliff amount
        let stream = create_test_stream(
            1000,  // start
            2000,  // end
            1000,  // deposited
            0,     // withdrawn
            100,   // period
            100,   // amount per period
            500,   // cliff (500 seconds)
            500,   // cliff amount (50%)
        );

        // Before cliff (at 1400): nothing vested, all locked
        let locked = stream.calculate_locked_at_timestamp(1400).unwrap();
        assert_eq!(locked, 1000);

        // Right at cliff (at 1500): cliff amount vested
        let locked = stream.calculate_locked_at_timestamp(1500).unwrap();
        assert_eq!(locked, 500);

        // After cliff: cliff + linear vesting
        // At 1700 (200 seconds after cliff = 2 periods): 500 + 200 = 700 vested, 300 locked
        let locked = stream.calculate_locked_at_timestamp(1700).unwrap();
        assert_eq!(locked, 300);
    }
}
