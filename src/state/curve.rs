use crate::error::CurveError;


#[derive(Debug)]
pub struct XYAmounts {
    pub x: u64,
    pub y: u64,
}

impl XYAmounts {
    // Get amount of X and Y to deposit from liquidity token amount
    pub fn xy_deposit_amounts_from_l(
        x: u64,
        y: u64,
        l: u64,
        a: u64,
        precision: u32,
    ) -> Result<XYAmounts, CurveError> {
        let ratio = (l as u128)
            .checked_add(a as u128)
            .ok_or(CurveError::Overflow)?
            .checked_mul(precision as u128)
            .ok_or(CurveError::Overflow)?
            .checked_div(l as u128)
            .ok_or(CurveError::Overflow)?;

        let deposit_x = (x as u128)
            .checked_mul(ratio)
            .ok_or(CurveError::Overflow)?
            .checked_div(precision as u128)
            .ok_or(CurveError::Overflow)?
            .checked_sub(x as u128)
            .ok_or(CurveError::Overflow)? as u64;

        let deposit_y = (y as u128)
            .checked_mul(ratio)
            .ok_or(CurveError::Overflow)?
            .checked_div(precision as u128)
            .ok_or(CurveError::Overflow)?
            .checked_sub(y as u128)
            .ok_or(CurveError::Overflow)? as u64;

        Ok(XYAmounts {
            x: deposit_x,
            y: deposit_y,
        })
    }
}
