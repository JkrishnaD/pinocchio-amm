use pinocchio::program_error::ProgramError;

impl From<PinocchioError> for ProgramError {
    fn from(e: PinocchioError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

pub enum PinocchioError {
    IdenticalTokenMints = 0x0,
    InvalidMintAmount = 0x1,
    InvalidOwner = 0x2,
    MathOverflow = 0x3,
    InvalidMintSupply = 0x4,
    InvalidAmount = 0x5,
    SlipageExceeded = 0x6,
    LessThanMinimum = 0x7,
    Expired = 0x8,
}

impl PinocchioError {
    pub fn description(&self) -> &'static str {
        match self {
            PinocchioError::IdenticalTokenMints => {
                "Cannot create a pool with identical token mints"
            }
            PinocchioError::InvalidMintAmount => "Invalid Mint Amount",
            PinocchioError::InvalidOwner => "Invalid Owner",
            PinocchioError::MathOverflow => "Math Overflow",
            PinocchioError::InvalidMintSupply => "Invalid Mint Supply",
            PinocchioError::InvalidAmount => "Invalid Amount",
            PinocchioError::SlipageExceeded => "Slippage Exceeded",
            PinocchioError::LessThanMinimum => "Amount is less than minimum",
            PinocchioError::Expired => "Withdrawal expired",
        }
    }
}
