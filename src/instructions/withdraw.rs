use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    sysvars::{clock::Clock, Sysvar},
};

use crate::{
    error::PinocchioError,
    instructions::{
        AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck,
        AssociatedTokenAccountInit, SignerAccount,
    },
};

pub struct WithdrawAccounts<'a> {
    pub user: &'a AccountInfo,

    pub mint_x: &'a AccountInfo,
    pub mint_y: &'a AccountInfo,

    pub mint_lp: &'a AccountInfo,

    pub vault_x: &'a AccountInfo,
    pub vault_y: &'a AccountInfo,

    pub user_x_ata: &'a AccountInfo,
    pub user_y_ata: &'a AccountInfo,
    pub user_lp_ata: &'a AccountInfo,

    pub config: &'a AccountInfo,

    pub token_program: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for WithdrawAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [user, mint_lp, vault_x, vault_y, mint_x, mint_y, user_x_ata, user_y_ata, user_lp_ata, config, token_program, system_program, _] =
            accounts
        else {
            return Err(ProgramError::InvalidAccountData);
        };

        SignerAccount::check(user)?;

        AssociatedTokenAccount::check(vault_x, config, mint_x)?;
        AssociatedTokenAccount::check(vault_y, config, mint_y)?;

        AssociatedTokenAccount::check(user_x_ata, user, mint_x)?;
        AssociatedTokenAccount::check(user_y_ata, user, mint_y)?;
        AssociatedTokenAccount::check(user_lp_ata, user, mint_lp)?;

        Ok(Self {
            user,
            mint_lp,
            vault_x,
            vault_y,
            mint_x,
            mint_y,
            user_x_ata,
            user_y_ata,
            user_lp_ata,
            config,
            token_program,
            system_program,
        })
    }
}

pub struct WithdrawInstructions {
    pub amount: u64,
    pub min_x: u64,
    pub min_y: u64,
    pub expiration: u64,
}

impl TryFrom<&[u8]> for WithdrawInstructions {
    type Error = ProgramError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() != size_of::<u64>() * 4 {
            return Err(ProgramError::InvalidInstructionData);
        }

        let amount = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let min_x = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let min_y = u64::from_le_bytes(data[16..24].try_into().unwrap());
        let expiration = u64::from_le_bytes(data[24..32].try_into().unwrap());

        if amount <= 0 || min_x <= 0 || min_y <= 0 {
            return Err(PinocchioError::LessThanMinimum.into());
        }

        if expiration > Clock::get()?.unix_timestamp as u64 {
            return Err(PinocchioError::Expired.into());
        }

        Ok(Self {
            amount,
            min_x,
            min_y,
            expiration,
        })
    }
}

pub struct Withdraw<'a> {
    pub accounts: WithdrawAccounts<'a>,
    pub instructions: WithdrawInstructions,
}

impl<'a> TryFrom<(&'a [AccountInfo], &[u8])> for Withdraw<'a> {
    type Error = ProgramError;

    fn try_from((accounts, data): (&'a [AccountInfo], &[u8])) -> Result<Self, Self::Error> {
        let accounts = WithdrawAccounts::try_from(accounts)?;
        let instructions = WithdrawInstructions::try_from(data)?;

        AssociatedTokenAccount::init_if_needed(
            accounts.user_x_ata,
            accounts.mint_x,
            accounts.user,
            accounts.user,
            accounts.system_program,
            accounts.token_program,
        )?;

        AssociatedTokenAccount::init_if_needed(
            accounts.user_y_ata,
            accounts.mint_y,
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
