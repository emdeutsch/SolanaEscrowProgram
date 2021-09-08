// inside instruction.rs
use solana_program::program_error::ProgramError;
use std::convert::TryInto;
use crate::error::AMMError::InvalidInstruction;

pub enum AMMInstruction {
    /// Initializes the AMM
    ///
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The Initiator Account
    /// 1. `[writable]` Temporary BUSD account that should be created prior to this instruction and owned by The Initiator Account
    /// 2. `[writable]` Temporary bStock account that should be created prior to this instruction and owned by The Initiator Account
    /// 3. `[writable]` Temporary bStockLQDY account that should be created prior to this instruction and owned by The Initiator Account
    /// 4. `[writable]` The AMM account, it will hold all necessary info about the trade.
    /// 5. `[]` The rent sysvar
    /// 6. `[]` The token program
    InitAMM {

    },

    /// Allows liquidity provider to provide {amount (in BUSD and bStock)} of liquidity
    ///  
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The liquidity provider's account
    /// 1. `[writable]` The BUSD token account owned by the liquidity provider's account
    /// 2. `[writable]` The bStock token account owned by the liquidity provider's account
    /// 3. `[writable]` The bStockLQDY token account owned by the liquidity provider's account
    /// 4. `[writable]` The PDA's BUSD token account
    /// 5. `[writable]` The PDA's bStock token account
    /// 6. `[writable]` The PDA's bStockLQDY token account
    /// 7. `[writable]` The AMM account holding the AMM info
    /// 8. `[]` The token program
    /// 9. `[]` The PDA account
    ProvLiquidity {
        busd_amount: u64,
        bstock_amount: u64,
    },

    /// Allows liquidity provider to claim {amount (in bStockLQDY)} of liquidity
    ///  
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The liquidity provider's account
    /// 1. `[writable]` The BUSD token account owned by the liquidity provider's account
    /// 2. `[writable]` The bStock token account owned by the liquidity provider's account
    /// 3. `[writable]` The bStockLQDY token account owned by the liquidity provider's account
    /// 4. `[writable]` The PDA's BUSD token account
    /// 5. `[writable]` The PDA's bStock token account
    /// 6. `[writable]` The PDA's bStockLQDY token account
    /// 7. `[writable]` The AMM account holding the AMM info
    /// 8. `[]` The token program
    /// 9. `[]` The PDA account
    ClaimLiquidity {
        amount: u64,
    },

    /// Allows user to trade {amount (in BUSD)} of BUSD in exchange for bStock
    ///  
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The user's account
    /// 1. `[writable]` The BUSD token account owned by the user's account
    /// 2. `[writable]` The bStock token account owned by the user's account
    /// 3. `[writable]` The PDA's BUSD token account
    /// 4. `[writable]` The PDA's bStock token account
    /// 5. `[writable]` The AMM account holding the AMM info
    /// 6. `[]` The token program
    /// 7. `[]` The PDA account
    TradeBUSD {
        amount: u64,
    },

    /// Allows user to trade {amount (in bStock)} of bStock in exchange for BUSD
    ///  
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The user's account
    /// 1. `[writable]` The BUSD token account owned by the user's account
    /// 2. `[writable]` The bStock token account owned by the user's account
    /// 3. `[writable]` The PDA's BUSD token account
    /// 4. `[writable]` The PDA's bStock token account
    /// 5. `[writable]` The AMM account holding the AMM info
    /// 6. `[]` The token program
    /// 7. `[]` The PDA account
    TradebStock {
        amount: u64,
    },

    /// Closes the AMM
    ///  
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The Initiator Account
    /// 1. `[writable]` The PDA's BUSD token account
    /// 2. `[writable]` The PDA's bStock token account
    /// 3. `[writable]` The PDA's bStockLQDY token account
    /// 4. `[writable]` The AMM account holding the AMM info
    /// 5. `[]` The token program
    /// 6. `[]` The PDA account
    CloseAMM {
        
    },
}

impl AMMInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;

        Ok(match tag {
            0 => Self::InitAMM {},
            1 => Self::ProvLiquidity {
                busd_amount: Self::unpack_amount(rest)?,
                bstock_amount: Self::unpack_second_amount(rest)?,
            },
            2 => Self::ClaimLiquidity {
                amount: Self::unpack_amount(rest)?,
            },
            3 => Self::TradeBUSD {
                amount: Self::unpack_amount(rest)?,
            },
            4 => Self::TradebStock {
                amount: Self::unpack_amount(rest)?,
            },
            5 => Self::CloseAMM {},
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(0..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }
    fn unpack_second_amount(input: &[u8]) -> Result<u64, ProgramError> {
        let amount = input
            .get(8..16)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        Ok(amount)
    }
}
