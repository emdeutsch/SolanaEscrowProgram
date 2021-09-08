// inside error.rs
use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum AMMError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,

    /// Invalid Initializer
    #[error("Invalid Initializer")]
    InvalidInitializer,

    /// Not Rent Exempt
    #[error("Not Rent Exempt")]
    NotRentExempt,

    /// InvalidRatio
    #[error("Invalid Ratio")]
    InvalidRatio,

    /// InvalidRatio
    #[error("Amount Overflow")]
    AmountOverflow,
}

impl From<AMMError> for ProgramError {
    fn from(e: AMMError) -> Self {
        ProgramError::Custom(e as u32)
    }
}