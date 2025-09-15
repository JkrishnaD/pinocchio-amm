use pinocchio::{account_info::AccountInfo, pubkey::Pubkey, ProgramResult};

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    Ok(())
}
