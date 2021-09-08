// inside state.rs
use solana_program::{
    program_pack::{IsInitialized, Pack, Sealed},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

pub struct AMM {
    pub is_initialized: bool,
    pub initializer_account_pubkey: Pubkey,
    pub busd_token_account_pubkey: Pubkey,
    pub bstock_token_account_pubkey: Pubkey,
    pub bstocklqdy_token_account_pubkey: Pubkey,
}

impl Sealed for AMM {}

impl IsInitialized for AMM {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Pack for AMM {
    const LEN: usize = 129;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, AMM::LEN];
        let (
            is_initialized,
            initializer_account_pubkey,
            busd_token_account_pubkey,
            bstock_token_account_pubkey,
            bstocklqdy_token_account_pubkey,
        ) = array_refs![src, 1, 32, 32, 32, 32];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(AMM {
            is_initialized, 
            initializer_account_pubkey: Pubkey::new_from_array(*initializer_account_pubkey),
            busd_token_account_pubkey: Pubkey::new_from_array(*busd_token_account_pubkey),
            bstock_token_account_pubkey: Pubkey::new_from_array(*bstock_token_account_pubkey),
            bstocklqdy_token_account_pubkey: Pubkey::new_from_array(*bstocklqdy_token_account_pubkey),
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, AMM::LEN];
        let (
            is_initialized_dst,
            initializer_account_pubkey_dst,
            busd_token_account_pubkey_dst,
            bstock_token_account_pubkey_dst,
            bstocklqdy_token_account_pubkey_dst,
        ) = mut_array_refs![dst, 1, 32, 32, 32, 32];

        let AMM {
            is_initialized,
            initializer_account_pubkey,
            busd_token_account_pubkey,
            bstock_token_account_pubkey,
            bstocklqdy_token_account_pubkey,
        } = self;

        is_initialized_dst[0] = *is_initialized as u8;
        initializer_account_pubkey_dst.copy_from_slice(initializer_account_pubkey.as_ref());
        busd_token_account_pubkey_dst.copy_from_slice(busd_token_account_pubkey.as_ref());
        bstock_token_account_pubkey_dst.copy_from_slice(bstock_token_account_pubkey.as_ref());
        bstocklqdy_token_account_pubkey_dst.copy_from_slice(bstocklqdy_token_account_pubkey.as_ref());
    }
}