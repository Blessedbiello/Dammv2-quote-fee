use anchor_lang::prelude::*;
use crate::error::ErrorCode;

/// Calculate pro-rata share using floor division
/// Returns: floor(total_amount * weight / total_weight)
pub fn calculate_pro_rata_share(
    total_amount: u64,
    individual_weight: u64,
    total_weight: u64,
) -> Result<u64> {
    if total_weight == 0 {
        return Ok(0);
    }

    let result = (total_amount as u128)
        .checked_mul(individual_weight as u128)
        .ok_or(ErrorCode::ArithmeticOverflow)?
        .checked_div(total_weight as u128)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    u64::try_from(result).map_err(|_| ErrorCode::ArithmeticOverflow.into())
}

/// Apply basis points to an amount
/// Returns: floor(amount * bps / 10000)
pub fn apply_bps(amount: u64, bps: u16) -> Result<u64> {
    let result = (amount as u128)
        .checked_mul(bps as u128)
        .ok_or(ErrorCode::ArithmeticOverflow)?
        .checked_div(10000u128)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    u64::try_from(result).map_err(|_| ErrorCode::ArithmeticOverflow.into())
}

/// Calculate f_locked as basis points
/// Returns: floor((locked_total / y0) * 10000)
pub fn calculate_f_locked_bps(locked_total: u64, y0: u64) -> Result<u64> {
    if y0 == 0 {
        return Err(ErrorCode::InvalidY0Amount.into());
    }

    let result = (locked_total as u128)
        .checked_mul(10000u128)
        .ok_or(ErrorCode::ArithmeticOverflow)?
        .checked_div(y0 as u128)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    u64::try_from(result).map_err(|_| ErrorCode::ArithmeticOverflow.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pro_rata_share() {
        // 100 total, weight 30/100 = 30
        assert_eq!(calculate_pro_rata_share(100, 30, 100).unwrap(), 30);

        // Floor division: 100 * 33 / 100 = 33 (not 33.33)
        assert_eq!(calculate_pro_rata_share(100, 33, 100).unwrap(), 33);

        // Zero weight
        assert_eq!(calculate_pro_rata_share(100, 0, 100).unwrap(), 0);

        // Zero total weight
        assert_eq!(calculate_pro_rata_share(100, 50, 0).unwrap(), 0);
    }

    #[test]
    fn test_apply_bps() {
        // 70% of 1000 = 700
        assert_eq!(apply_bps(1000, 7000).unwrap(), 700);

        // 100% of 1000 = 1000
        assert_eq!(apply_bps(1000, 10000).unwrap(), 1000);

        // 0% of 1000 = 0
        assert_eq!(apply_bps(1000, 0).unwrap(), 0);

        // Floor: 33.33% of 100 = 33 (not 33.33)
        assert_eq!(apply_bps(100, 3333).unwrap(), 33);
    }

    #[test]
    fn test_f_locked_bps() {
        // 700 locked out of 1000 total = 7000 bps (70%)
        assert_eq!(calculate_f_locked_bps(700, 1000).unwrap(), 7000);

        // All locked = 10000 bps (100%)
        assert_eq!(calculate_f_locked_bps(1000, 1000).unwrap(), 10000);

        // None locked = 0 bps
        assert_eq!(calculate_f_locked_bps(0, 1000).unwrap(), 0);

        // Half locked = 5000 bps (50%)
        assert_eq!(calculate_f_locked_bps(500, 1000).unwrap(), 5000);
    }
}
