use pinocchio::{
    account_info::AccountInfo, instruction::Seed, program_error::ProgramError, ProgramResult,
};

use crate::{
    instructions::{
        AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountInit, MintInterface,
        ProgramAccount, ProgramAccountInit, SignerAccount,
    },
    state::Config,
};

pub struct InitializeConfigAccounts<'a> {
    pub authority: &'a AccountInfo,
    pub config: &'a AccountInfo,

    pub mint_x: &'a AccountInfo,
    pub mint_y: &'a AccountInfo,

    pub vault_x: &'a AccountInfo,
    pub vault_y: &'a AccountInfo,

    pub lp_mint: &'a AccountInfo,

    pub token_program: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub associated_token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeConfigAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, config, mint_x, mint_y, vault_x, vault_y, lp_mint, token_program, system_program, associated_token_program] =
            accounts
        else {
            return Err(ProgramError::InvalidAccountData);
        };

        SignerAccount::check(authority)?;
        MintInterface::check(mint_x)?;
        MintInterface::check(mint_y)?;

        if mint_x.key() == mint_y.key() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            authority,
            config,
            mint_x,
            mint_y,
            vault_x,
            vault_y,
            lp_mint,
            token_program,
            system_program,
            associated_token_program,
        })
    }
}

pub struct InitializeConfigInstruction {
    pub fee: u16,
    pub config_bump: u8,
}

impl<'a> TryFrom<&'a [u8]> for InitializeConfigInstruction {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() < 2 {
            return Err(ProgramError::InvalidAccountData);
        };

        let fee = u16::from_le_bytes(data[0..2].try_into().unwrap());
        let config_bump = u8::from_le_bytes([data[2]]);

        if fee > 1000 {
            return Err(ProgramError::InvalidAccountData);
        };
        Ok(Self { fee, config_bump })
    }
}

pub struct InitializeConfig<'a> {
    pub accounts: InitializeConfigAccounts<'a>,
    pub instruction: InitializeConfigInstruction,
}

impl<'a> TryFrom<(&'a [AccountInfo], &'a [u8])> for InitializeConfig<'a> {
    type Error = ProgramError;

    fn try_from(value: (&'a [AccountInfo], &'a [u8])) -> Result<Self, Self::Error> {
        let accounts = InitializeConfigAccounts::try_from(value.0)?;
        let instruction = InitializeConfigInstruction::try_from(value.1)?;

        // seeds for the config account
        let config_bindings = instruction.config_bump.to_le_bytes();
        let config_seeds = [Seed::from(b"config"), Seed::from(&config_bindings)];

        // creation of the config account
        ProgramAccount::init::<Config>(
            accounts.authority,
            accounts.config,
            &config_seeds,
            Config::LEN,
        )?;

        // seeds for the lp mint account
        let lp_mint_seeds = [
            Seed::from(b"lp_mint"),
            Seed::from(accounts.config.key().as_ref()),
        ];

        // creation of the lp mint account
        ProgramAccount::init::<pinocchio_token::state::Mint>(
            accounts.authority,
            accounts.lp_mint,
            &lp_mint_seeds,
            pinocchio_token::state::Mint::LEN,
        )?;

        // creation of vault_x associated token account
        AssociatedTokenAccount::init(
            accounts.vault_x,
            accounts.mint_x,
            accounts.authority,
            accounts.config,
            accounts.system_program,
            accounts.token_program,
        )?;

        // creation of vault_y associated token account
        AssociatedTokenAccount::init(
            accounts.vault_y,
            accounts.mint_y,
            accounts.authority,
            accounts.config,
            accounts.system_program,
            accounts.token_program,
        )?;

        Ok(Self {
            accounts,
            instruction,
        })
    }
}

impl<'a> InitializeConfig<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&self) -> ProgramResult {
        // get the config account mutable data
        let mut config_data = Config::load_mut(self.accounts.config)?;

        // set the config account data
        config_data.set_inner(
            *self.accounts.authority.key(),
            *self.accounts.mint_x.key(),
            *self.accounts.mint_y.key(),
            *self.accounts.vault_x.key(),
            *self.accounts.vault_y.key(),
            *self.accounts.lp_mint.key(),
            self.instruction.fee,
            self.instruction.config_bump,
        )?;

        Ok(())
    }
}
