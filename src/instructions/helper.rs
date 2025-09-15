use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::find_program_address,
    sysvars::{rent::Rent, Sysvar},
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::state::Mint;

pub trait AccountCheck {
    fn check(account: &AccountInfo) -> Result<(), ProgramError>;
}

pub struct SignerAccount;
// account checks for the signer
impl AccountCheck for SignerAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_signer() {
            return Err(ProgramError::MissingRequiredSignature);
        }
        Ok(())
    }
}

pub struct MintInterface;
// mint accounts checks
impl AccountCheck for MintInterface {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if account.data_len() != Mint::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

pub struct TokenAccount;
// token accounts checks
impl AccountCheck for TokenAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_owned_by(&pinocchio_token::ID) {
            return Err(ProgramError::IllegalOwner.into());
        }

        if account.data_len() != pinocchio_token::state::TokenAccount::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

pub trait ProgramAccountInit {
    fn init<'a, T: Sized>(
        payer: &AccountInfo,
        account: &AccountInfo,
        seeds: &[Seed<'a>],
        space: usize,
    ) -> Result<(), ProgramError>;
}

pub struct ProgramAccount;

impl ProgramAccountInit for ProgramAccount {
    fn init<'a, T: Sized>(
        payer: &AccountInfo,
        account: &AccountInfo,
        seeds: &[Seed<'a>],
        space: usize,
    ) -> Result<(), ProgramError> {
        // get the lamports for the rent excempt
        let rent_excempt = Rent::get()?.minimum_balance(space);

        // creating the signer from the seeds
        let signer = [Signer::from(seeds)];

        // creating the account with the data
        CreateAccount {
            from: payer,
            to: account,
            lamports: rent_excempt,
            space: space as u64,
            owner: account.owner(),
        }
        .invoke_signed(&signer)?;
        Ok(())
    }
}

// ata checks
pub trait AssociatedTokenAccountCheck {
    fn check(
        account: &AccountInfo,
        authority: &AccountInfo,
        mint: &AccountInfo,
    ) -> Result<(), ProgramError>;
}

// ata init's
pub trait AssociatedTokenAccountInit {
    fn init(
        ata: &AccountInfo,
        mint: &AccountInfo,
        authority: &AccountInfo,
        owner: &AccountInfo,
        system_program: &AccountInfo,
        token_program: &AccountInfo,
    ) -> Result<(), ProgramError>;

    fn init_if_needed(
        ata: &AccountInfo,
        mint: &AccountInfo,
        authority: &AccountInfo,
        owner: &AccountInfo,
        system_program: &AccountInfo,
        token_program: &AccountInfo,
    ) -> Result<(), ProgramError>;
}

pub struct AssociatedTokenAccount;

impl AssociatedTokenAccountCheck for AssociatedTokenAccount {
    fn check(
        account: &AccountInfo,
        authority: &AccountInfo,
        mint: &AccountInfo,
    ) -> Result<(), ProgramError> {
        TokenAccount::check(account)?;

        let seeds: &[&[u8]] = &[authority.key(), &pinocchio_token::ID, mint.key()];

        if find_program_address(seeds, &pinocchio_associated_token_account::ID)
            .0
            .ne(account.key())
        {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

impl AssociatedTokenAccountInit for AssociatedTokenAccount {
    fn init(
        ata: &AccountInfo,
        mint: &AccountInfo,
        authority: &AccountInfo,
        owner: &AccountInfo,
        system_program: &AccountInfo,
        token_program: &AccountInfo,
    ) -> Result<(), ProgramError> {
        pinocchio_associated_token_account::instructions::Create {
            account: ata,
            funding_account: authority,
            mint: mint,
            wallet: owner,
            system_program,
            token_program,
        }
        .invoke()?;
        Ok(())
    }

    fn init_if_needed(
        ata: &AccountInfo,
        mint: &AccountInfo,
        authority: &AccountInfo,
        owner: &AccountInfo,
        system_program: &AccountInfo,
        token_program: &AccountInfo,
    ) -> Result<(), ProgramError> {
        // checking the ata is initialized or not
        match Self::check(ata, authority, mint) {
            Ok(_) => Ok(()),
            Err(_) => Self::init(ata, mint, authority, owner, system_program, token_program),
        }?;
        Ok(())
    }
}
