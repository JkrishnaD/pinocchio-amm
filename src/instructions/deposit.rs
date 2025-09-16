use core::cmp;

use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::find_program_address,
    ProgramResult,
};
use pinocchio_token::{
    instructions::{MintTo, Transfer},
    state::TokenAccount,
};

use crate::{
    error::PinocchioError,
    instructions::{
        AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck,
        AssociatedTokenAccountInit, MintInterface, SignerAccount,
    },
    state::Config,
};

pub struct DepositAccounts<'a> {
    pub user: &'a AccountInfo,

    pub mint_x: &'a AccountInfo,
    pub mint_y: &'a AccountInfo,
    pub lp_mint: &'a AccountInfo,

    pub config: &'a AccountInfo,

    pub vault_x: &'a AccountInfo,
    pub vault_y: &'a AccountInfo,
    pub vault_lp: &'a AccountInfo,

    pub user_x_ata: &'a AccountInfo,
    pub user_y_ata: &'a AccountInfo,
    pub user_lp_ata: &'a AccountInfo,

    pub token_program: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub associated_token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for DepositAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [user, mint_x, mint_y, lp_mint, config, vault_x, vault_y, user_lp_ata, user_x_ata, user_y_ata, vault_lp, token_program, system_program, associated_token_program] =
            accounts
        else {
            return Err(ProgramError::InvalidAccountData);
        };

        // account checks
        SignerAccount::check(user)?;
        MintInterface::check(mint_x)?;
        MintInterface::check(mint_y)?;

        AssociatedTokenAccount::check(user_x_ata, user, mint_x)?;
        AssociatedTokenAccount::check(user_y_ata, user, mint_y)?;
        AssociatedTokenAccount::check(user_lp_ata, user, lp_mint)?;
        AssociatedTokenAccount::check(vault_lp, vault_x, lp_mint)?;
        AssociatedTokenAccount::check(vault_lp, vault_y, lp_mint)?;

        let seeds = &[b"lp_mint", config.key().as_ref()];
        let (expected_lp_mint, _) = find_program_address(seeds, &crate::ID);

        if expected_lp_mint != *lp_mint.key() {
            return Err(ProgramError::InvalidAccountData);
        }

        if mint_x.key() == mint_y.key() {
            return Err(PinocchioError::IdenticalTokenMints.into());
        }

        Ok(Self {
            user,
            mint_x,
            mint_y,
            lp_mint,
            config,
            vault_x,
            vault_y,
            user_lp_ata,
            user_x_ata,
            user_y_ata,
            vault_lp,
            token_program,
            system_program,
            associated_token_program,
        })
    }
}

pub struct DepositInstructions {
    pub mint_x: u64,
    pub mint_y: u64,
    pub min_lp_amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for DepositInstructions {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != 24 {
            return Err(ProgramError::InvalidInstructionData);
        };

        let mint_x = u64::from_le_bytes([
            data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
        ]);
        let mint_y = u64::from_le_bytes([
            data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15],
        ]);
        let min_lp_amount = u64::from_le_bytes([
            data[16], data[17], data[18], data[19], data[20], data[21], data[22], data[23],
        ]);

        if mint_x == 0 || mint_y == 0 {
            return Err(PinocchioError::InvalidMintAmount.into());
        }

        Ok(Self {
            mint_x,
            mint_y,
            min_lp_amount,
        })
    }
}

pub struct Deposit<'a> {
    pub accounts: DepositAccounts<'a>,
    pub instructions: DepositInstructions,
}

impl<'a> TryFrom<(&'a [AccountInfo], &'a [u8])> for Deposit<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountInfo], &'a [u8])) -> Result<Self, Self::Error> {
        let accounts = DepositAccounts::try_from(accounts)?;
        let instructions = DepositInstructions::try_from(data)?;

        // user lp ata account creation if needed
        AssociatedTokenAccount::init_if_needed(
            accounts.user_lp_ata,
            accounts.lp_mint,
            accounts.user,
            accounts.user,
            accounts.system_program,
            accounts.token_program,
        )?;

        Ok(Self {
            accounts,
            instructions,
        })
    }
}

impl<'a> Deposit<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;
    pub fn process(&self) -> ProgramResult {
        let config = Config::load(self.accounts.config)?;

        let config_bump = config.config_bump();

        // seeds derivation
        let config_bindings = config_bump.to_le_bytes();
        let config_seeds = [b"config", config_bindings.as_ref()];
        let (expected_config, _) = find_program_address(&config_seeds, &crate::ID);

        let lp_mint_seeds = [b"lp_mint", self.accounts.config.key().as_ref()];
        let (expected_lp_mint, _) = find_program_address(&lp_mint_seeds, &crate::ID);

        // PDA's validation
        if expected_config != *self.accounts.config.key() {
            return Err(PinocchioError::InvalidConfig.into());
        }

        if expected_lp_mint != *self.accounts.lp_mint.key() {
            return Err(PinocchioError::InvalidLpMint.into());
        }

        // getting the vault datas
        let vault_x_data = self.accounts.vault_x.try_borrow_data()?;
        let vault_x = unsafe { TokenAccount::from_bytes_unchecked(&vault_x_data) };

        let vault_y_data = self.accounts.vault_y.try_borrow_data()?;
        let vault_y = unsafe { TokenAccount::from_bytes_unchecked(&vault_y_data) };

        let vault_lp_data = self.accounts.vault_lp.try_borrow_data()?;
        let vault_lp = unsafe { TokenAccount::from_bytes_unchecked(&vault_lp_data) };

        if vault_x.owner() != self.accounts.config.key()
            || vault_y.owner() != self.accounts.config.key()
        {
            return Err(PinocchioError::InvalidOwner.into());
        }

        if vault_x.mint() != self.accounts.mint_x.key()
            || vault_y.mint() != self.accounts.mint_y.key()
        {
            return Err(ProgramError::InvalidAccountData);
        };

        let reserve_mint_x = vault_x.amount();
        let reserve_mint_y = vault_y.amount();

        let lp_supply = vault_lp.amount();

        let lp_mint_tokens_supply = if reserve_mint_x == 0 && reserve_mint_y == 0 {
            let product = (self.instructions.mint_x as u128)
                .checked_mul(self.instructions.mint_y as u128)
                .ok_or_else(|| PinocchioError::MathOverflow)?;

            if product == 0 {
                return Err(PinocchioError::InvalidMintSupply.into());
            }

            let sqrt_result = product.isqrt() as u64;

            if sqrt_result < 1000 {
                return Err(PinocchioError::InvalidMintSupply.into());
            }

            sqrt_result
        } else {
            if reserve_mint_x == 0 || reserve_mint_y == 0 || lp_supply == 0 {
                return Err(PinocchioError::InvalidMintSupply.into());
            };

            let lp_from_x = (self.instructions.mint_x as u128)
                .checked_mul(lp_supply as u128)
                .ok_or_else(|| PinocchioError::MathOverflow)?
                .checked_div(reserve_mint_x as u128)
                .ok_or_else(|| PinocchioError::MathOverflow)? as u64;

            let lp_from_y = (self.instructions.mint_y as u128)
                .checked_mul(lp_supply as u128)
                .ok_or_else(|| PinocchioError::MathOverflow)?
                .checked_div(reserve_mint_y as u128)
                .ok_or_else(|| PinocchioError::MathOverflow)? as u64;

            cmp::min(lp_from_x, lp_from_y)
        };

        if lp_mint_tokens_supply == 0 {
            return Err(PinocchioError::InvalidAmount.into());
        }

        if lp_mint_tokens_supply < self.instructions.min_lp_amount {
            return Err(PinocchioError::SlipageExceeded.into());
        }

        Transfer {
            from: self.accounts.user_x_ata,
            to: self.accounts.vault_x,
            amount: self.instructions.mint_x,
            authority: self.accounts.user,
        }
        .invoke()?;

        Transfer {
            from: self.accounts.user_y_ata,
            to: self.accounts.vault_y,
            amount: self.instructions.mint_y,
            authority: self.accounts.user,
        }
        .invoke()?;

        MintTo {
            account: self.accounts.lp_mint,
            mint: self.accounts.lp_mint,
            amount: lp_mint_tokens_supply,
            mint_authority: self.accounts.config,
        }
        .invoke()?;
        Ok(())
    }
}
