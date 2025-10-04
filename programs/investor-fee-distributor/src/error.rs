use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Pool config does not have quote-only fee collection enabled (collectFeeMode != 1)")]
    PoolNotQuoteOnlyFees,

    #[msg("Base token fees detected in treasury - distribution aborted to enforce quote-only")]
    BaseFeesDetected,

    #[msg("Too early to crank next day - must wait 24 hours since last window start")]
    TooEarlyForNextDay,

    #[msg("Current time is outside the valid 24-hour daily window")]
    OutsideWindow,

    #[msg("Day is already finalized - no more distributions allowed")]
    DayAlreadyFinalized,

    #[msg("Daily cap reached - cannot distribute more fees today")]
    DailyCapReached,

    #[msg("Invalid investor page data provided")]
    InvalidInvestorPage,

    #[msg("Streamflow account data mismatch or invalid")]
    StreamflowAccountMismatch,

    #[msg("Arithmetic overflow in distribution calculation")]
    ArithmeticOverflow,

    #[msg("Arithmetic underflow in calculation")]
    ArithmeticUnderflow,

    #[msg("Invalid token mint - does not match expected quote or base mint")]
    InvalidTokenMint,

    #[msg("Position does not exist or is invalid")]
    InvalidPosition,

    #[msg("Policy configuration is invalid")]
    InvalidPolicy,

    #[msg("No fees available to distribute")]
    NoFeesAvailable,

    #[msg("Total pages must be greater than zero")]
    InvalidTotalPages,

    #[msg("Investor fee share basis points exceeds maximum (10000)")]
    InvalidFeeShareBps,

    #[msg("Y0 total streamed amount cannot be zero")]
    InvalidY0Amount,
}
