use pinocchio::{
    account_info::{AccountInfo, Ref, RefMut},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::error::CurveError;

#[repr(C)]
pub struct Config {
    authority: Pubkey,
    mint_x: Pubkey,
    mint_y: Pubkey,
    mint_x_vault: Pubkey,
    mint_y_vault: Pubkey,
    lp_mint: Pubkey,
    fee: u16,
    config_bump: u8,
}

#[repr(u8)]
pub enum AmmState {
    Uninitialized = 0u8,
    Initialized = 1u8,
    Disabled = 2u8,
    WithdrawOnly = 3u8,
}

impl Config {
    pub const LEN: usize = size_of::<Self>();

    // inline always attribute rather than adding the function call to the cll stack
    // it adds the function code to the call stack which eliminate the overhead function call
    #[inline(always)]
    pub fn load(account_info: &AccountInfo) -> Result<Ref<Self>, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if account_info.owner().ne(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(Ref::map(account_info.try_borrow_data()?, |data| unsafe {
            Self::from_bytes_unchecked(data)
        }))
    }

    #[inline(always)]
    pub unsafe fn load_unchecked(account_info: &AccountInfo) -> Result<&Self, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if account_info.owner().ne(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(Self::from_bytes_unchecked(
            account_info.borrow_data_unchecked(),
        ))
    }

    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const Config)
    }

    #[inline(always)]
    pub fn load_mut(account_info: &AccountInfo) -> Result<RefMut<Self>, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if account_info.owner().ne(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(RefMut::map(
            account_info.try_borrow_mut_data()?,
            |data| unsafe { Self::from_bytes_unchecked_mut(data) },
        ))
    }

    #[inline(always)]
    pub unsafe fn from_bytes_unchecked_mut(bytes: &mut [u8]) -> &mut Self {
        &mut *(bytes.as_mut_ptr() as *mut Config)
    }

    pub fn config_bump(&self) -> u8 {
        self.config_bump
    }

    pub fn set_inner(
        &mut self,
        authority: Pubkey,
        mint_x: Pubkey,
        mint_y: Pubkey,
        mint_x_vault: Pubkey,
        mint_y_vault: Pubkey,
        lp_mint: Pubkey,
        fee: u16,
        config_bump: u8,
    ) -> Result<(), ProgramError> {
        self.authority = authority;
        self.mint_x = mint_x;
        self.mint_y = mint_y;
        self.mint_x_vault = mint_x_vault;
        self.mint_y_vault = mint_y_vault;
        self.lp_mint = lp_mint;
        self.fee = fee;
        self.config_bump = config_bump;
        Ok(())
    }

    pub fn has_authority(&self) -> Option<Pubkey> {
        if self.authority != Pubkey::default() {
            Some(self.authority)
        } else {
            None
        }
    }
}

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
