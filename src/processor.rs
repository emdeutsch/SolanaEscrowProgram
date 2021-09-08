use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

use spl_token::state::Account as TokenAccount;

use crate::{error::AMMError, instruction::AMMInstruction, state::AMM};

pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = AMMInstruction::unpack(instruction_data)?;

        match instruction {
            AMMInstruction::InitAMM {  } => {
                msg!("Instruction: InitAMM");
                Self::process_init_amm(accounts, program_id)
            }
            AMMInstruction::ProvLiquidity { busd_amount, bstock_amount } => {
                msg!("Instruction: ProvLiquidity");
                Self::process_provide_liquidity(accounts, busd_amount, bstock_amount, program_id)
            }
            AMMInstruction::ClaimLiquidity { amount } => {
                msg!("Instruction: ClaimLiquidity");
                Self::process_claim_liquidity(accounts, amount, program_id)
            }
            AMMInstruction::TradeBUSD { amount } => {
                msg!("Instruction: TradeBUSD");
                Self::process_trade_busd(accounts, amount, program_id)
            }
            AMMInstruction::TradebStock { amount } => {
                msg!("Instruction: TradebStock");
                Self::process_trade_bstock(accounts, amount, program_id)
            }
            AMMInstruction::CloseAMM {} => {
                msg!("Instruction: CloseAMM");
                Self::process_close_amm(accounts, program_id)
            }
        }
    }

    fn process_init_amm(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer_account = next_account_info(account_info_iter)?;

        if !initializer_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let temp_busd_token_account = next_account_info(account_info_iter)?;
        let temp_bstock_token_account = next_account_info(account_info_iter)?;
        let temp_bstocklqdy_token_account = next_account_info(account_info_iter)?;

        let amm_account = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        if !rent.is_exempt(amm_account.lamports(), amm_account.data_len()) {
            return Err(AMMError::NotRentExempt.into());
        }

        let mut amm_state = AMM::unpack_unchecked(&amm_account.data.borrow())?;
        if amm_state.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        amm_state.is_initialized = true;
        amm_state.initializer_account_pubkey = *initializer_account.key;
        amm_state.busd_token_account_pubkey = *temp_busd_token_account.key;
        amm_state.bstock_token_account_pubkey = *temp_bstock_token_account.key;
        amm_state.bstocklqdy_token_account_pubkey = *temp_bstocklqdy_token_account.key;

        AMM::pack(amm_state, &mut amm_account.data.borrow_mut())?;
        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"bravv"], program_id);

        let token_program = next_account_info(account_info_iter)?;

        let busd_owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            temp_busd_token_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer_account.key,
            &[&initializer_account.key],
        )?;
        let bstock_owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            temp_busd_token_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer_account.key,
            &[&initializer_account.key],
        )?;
        let bstocklqdy_owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            temp_busd_token_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer_account.key,
            &[&initializer_account.key],
        )?;

        msg!("Calling the token program to transfer token account ownership...");
        invoke(
            &busd_owner_change_ix,
            &[
                temp_busd_token_account.clone(),
                initializer_account.clone(),
                token_program.clone(),
            ],
        )?;
        invoke(
            &bstock_owner_change_ix,
            &[
                temp_busd_token_account.clone(),
                initializer_account.clone(),
                token_program.clone(),
            ],
        )?;
        invoke(
            &bstocklqdy_owner_change_ix,
            &[
                temp_busd_token_account.clone(),
                initializer_account.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }

    fn process_provide_liquidity(
        accounts: &[AccountInfo],
        busd_amount: u64,
        bstock_amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        
        let liquidity_provider_account = next_account_info(account_info_iter)?;
        if !liquidity_provider_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let liquidity_provider_bstocklqdy_token_account = next_account_info(account_info_iter)?;
        let liquidity_provider_busd_token_account = next_account_info(account_info_iter)?;
        let liquidity_provider_bstock_token_account = next_account_info(account_info_iter)?;

        let pda_busd_token_account = next_account_info(account_info_iter)?;
        let pda_bstock_token_account = next_account_info(account_info_iter)?;
        let pda_bstocklqdy_token_account = next_account_info(account_info_iter)?;

        let pda_busd_token_account_info =
            TokenAccount::unpack(&pda_busd_token_account.data.borrow())?;
        let pda_bstock_token_account_info =
            TokenAccount::unpack(&pda_bstock_token_account.data.borrow())?;
        // let pda_bstocklqdy_token_account_info =
        //     TokenAccount::unpack(&pda_bstocklqdy_token_account.data.borrow())?;

        let amm_ratio = pda_busd_token_account_info.amount / pda_bstock_token_account_info.amount;
        let provider_ratio = bstock_amount / busd_amount;
        
        if amm_ratio != provider_ratio {
            return Err(AMMError::InvalidRatio.into());
        }

        let (pda, bump_seed) = Pubkey::find_program_address(&[b"bravv"], program_id);

        let amm_account = next_account_info(account_info_iter)?;
        let amm_info = AMM::unpack(&amm_account.data.borrow())?;
        if amm_info.busd_token_account_pubkey != *pda_busd_token_account.key || amm_info.bstock_token_account_pubkey != *pda_bstock_token_account.key || amm_info.bstocklqdy_token_account_pubkey != *pda_bstocklqdy_token_account.key {
            return Err(ProgramError::InvalidAccountData);
        }
        let token_program = next_account_info(account_info_iter)?;
        let transfer_busd_to_amm = spl_token::instruction::transfer(
            token_program.key,
            liquidity_provider_busd_token_account.key,
            pda_busd_token_account.key,
            liquidity_provider_account.key,
            &[&liquidity_provider_account.key],
            busd_amount,
        )?;
        let transfer_bstock_to_amm = spl_token::instruction::transfer(
            token_program.key,
            liquidity_provider_bstock_token_account.key,
            pda_bstock_token_account.key,
            liquidity_provider_account.key,
            &[&liquidity_provider_account.key],
            bstock_amount,
        )?;
        msg!("Calling the token program to transfer tokens to the AMM...");
        invoke(
            &transfer_busd_to_amm,
            &[
                liquidity_provider_busd_token_account.clone(),
                pda_busd_token_account.clone(),
                liquidity_provider_account.clone(),
                token_program.clone(),
            ],
        )?;
        invoke(
            &transfer_bstock_to_amm,
            &[
                liquidity_provider_bstock_token_account.clone(),
                pda_bstock_token_account.clone(),
                liquidity_provider_account.clone(),
                token_program.clone(),
            ],
        )?;

        let pda_account = next_account_info(account_info_iter)?;

        let transfer_bstocklqdy_to_liquidity_provider = spl_token::instruction::transfer(
            token_program.key,
            pda_bstocklqdy_token_account.key,
            liquidity_provider_bstocklqdy_token_account.key,
            &pda,
            &[&pda],
            busd_amount,
        )?;
        msg!("Calling the token program to transfer bStockLQDY to the liquidity provider...");
        invoke_signed(
            &transfer_bstocklqdy_to_liquidity_provider,
            &[
                pda_bstocklqdy_token_account.clone(),
                liquidity_provider_bstocklqdy_token_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"bravv"[..], &[bump_seed]]],
        )?;

        Ok(())
    }

    fn process_claim_liquidity(
        accounts: &[AccountInfo],
        bstocklqdy_amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        
        let liquidity_provider_account = next_account_info(account_info_iter)?;
        if !liquidity_provider_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let liquidity_provider_busd_token_account = next_account_info(account_info_iter)?;
        let liquidity_provider_bstock_token_account = next_account_info(account_info_iter)?;
        let liquidity_provider_bstocklqdy_token_account = next_account_info(account_info_iter)?;

        let pda_busd_token_account = next_account_info(account_info_iter)?;
        let pda_bstock_token_account = next_account_info(account_info_iter)?;
        let pda_bstocklqdy_token_account = next_account_info(account_info_iter)?;

        let pda_busd_token_account_info =
            TokenAccount::unpack(&pda_busd_token_account.data.borrow())?;
        let pda_bstock_token_account_info =
            TokenAccount::unpack(&pda_bstock_token_account.data.borrow())?;
        // let pda_bstocklqdy_token_account_info =
        //     TokenAccount::unpack(&pda_bstocklqdy_token_account.data.borrow())?;

        let amm_ratio = pda_busd_token_account_info.amount / pda_bstock_token_account_info.amount;
        let busd_amount = bstocklqdy_amount;
        let bstock_amount = busd_amount * amm_ratio;

        let (pda, bump_seed) = Pubkey::find_program_address(&[b"bravv"], program_id);

        let amm_account = next_account_info(account_info_iter)?;
        let amm_info = AMM::unpack(&amm_account.data.borrow())?;
        if amm_info.busd_token_account_pubkey != *pda_busd_token_account.key || amm_info.bstock_token_account_pubkey != *pda_bstock_token_account.key || amm_info.bstocklqdy_token_account_pubkey != *pda_bstocklqdy_token_account.key {
            return Err(ProgramError::InvalidAccountData);
        }
        let token_program = next_account_info(account_info_iter)?;
        let transfer_bstocklqdy_to_amm = spl_token::instruction::transfer(
            token_program.key,
            liquidity_provider_bstocklqdy_token_account.key,
            pda_bstocklqdy_token_account.key,
            liquidity_provider_account.key,
            &[&liquidity_provider_account.key],
            bstocklqdy_amount,
        )?;
        msg!("Calling the token program to transfer tokens to the AMM...");
        invoke(
            &transfer_bstocklqdy_to_amm,
            &[
                liquidity_provider_bstocklqdy_token_account.clone(),
                pda_busd_token_account.clone(),
                liquidity_provider_account.clone(),
                token_program.clone(),
            ],
        )?;

        let pda_account = next_account_info(account_info_iter)?;

        let transfer_busd_to_liquidity_provider = spl_token::instruction::transfer(
            token_program.key,
            pda_bstocklqdy_token_account.key,
            liquidity_provider_busd_token_account.key,
            &pda,
            &[&pda],
            busd_amount,
        )?;
        let transfer_bstock_to_liquidity_provider = spl_token::instruction::transfer(
            token_program.key,
            pda_bstock_token_account.key,
            liquidity_provider_bstock_token_account.key,
            &pda,
            &[&pda],
            bstock_amount,
        )?;
        msg!("Calling the token program to transfer bStockLQDY to the liquidity provider...");
        invoke_signed(
            &transfer_busd_to_liquidity_provider,
            &[
                pda_busd_token_account.clone(),
                liquidity_provider_busd_token_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"bravv"[..], &[bump_seed]]],
        )?;
        invoke_signed(
            &transfer_bstock_to_liquidity_provider,
            &[
                pda_bstock_token_account.clone(),
                liquidity_provider_bstock_token_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"bravv"[..], &[bump_seed]]],
        )?;

        Ok(())
    }

    fn process_trade_busd(
        accounts: &[AccountInfo],
        busd_amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user_account = next_account_info(account_info_iter)?;
        if !user_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let user_busd_token_account = next_account_info(account_info_iter)?;
        let user_bstock_token_account = next_account_info(account_info_iter)?;

        let pda_busd_token_account = next_account_info(account_info_iter)?;
        let pda_bstock_token_account = next_account_info(account_info_iter)?;

        let pda_busd_token_account_info =
            TokenAccount::unpack(&pda_busd_token_account.data.borrow())?;
        let pda_bstock_token_account_info =
            TokenAccount::unpack(&pda_bstock_token_account.data.borrow())?;
        // let pda_bstocklqdy_token_account_info =
        //     TokenAccount::unpack(&pda_bstocklqdy_token_account.data.borrow())?;

        let amm_ratio = pda_busd_token_account_info.amount / pda_bstock_token_account_info.amount;
        let bstock_amount = busd_amount * amm_ratio;

        let (pda, bump_seed) = Pubkey::find_program_address(&[b"bravv"], program_id);

        let amm_account = next_account_info(account_info_iter)?;
        let amm_info = AMM::unpack(&amm_account.data.borrow())?;
        if amm_info.busd_token_account_pubkey != *pda_busd_token_account.key || amm_info.bstock_token_account_pubkey != *pda_bstock_token_account.key {
            return Err(ProgramError::InvalidAccountData);
        }
        let token_program = next_account_info(account_info_iter)?;
        let transfer_busd_to_amm = spl_token::instruction::transfer(
            token_program.key,
            user_busd_token_account.key,
            pda_busd_token_account.key,
            user_account.key,
            &[&user_account.key],
            busd_amount,
        )?;
        msg!("Calling the token program to transfer tokens to the AMM...");
        invoke(
            &transfer_busd_to_amm,
            &[
                user_busd_token_account.clone(),
                pda_busd_token_account.clone(),
                user_account.clone(),
                token_program.clone(),
            ],
        )?;

        let pda_account = next_account_info(account_info_iter)?;

        let transfer_bstock_to_user = spl_token::instruction::transfer(
            token_program.key,
            pda_bstock_token_account.key,
            user_bstock_token_account.key,
            &pda,
            &[&pda],
            bstock_amount,
        )?;
        msg!("Calling the token program to transfer bStock to the user...");
        invoke_signed(
            &transfer_bstock_to_user,
            &[
                pda_bstock_token_account.clone(),
                user_bstock_token_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"bravv"[..], &[bump_seed]]],
        )?;

        Ok(())
    }

    fn process_trade_bstock(
        accounts: &[AccountInfo],
        busd_amount: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let user_account = next_account_info(account_info_iter)?;
        if !user_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
        let user_busd_token_account = next_account_info(account_info_iter)?;
        let user_bstock_token_account = next_account_info(account_info_iter)?;

        let pda_busd_token_account = next_account_info(account_info_iter)?;
        let pda_bstock_token_account = next_account_info(account_info_iter)?;

        let pda_busd_token_account_info =
            TokenAccount::unpack(&pda_busd_token_account.data.borrow())?;
        let pda_bstock_token_account_info =
            TokenAccount::unpack(&pda_bstock_token_account.data.borrow())?;
        // let pda_bstocklqdy_token_account_info =
        //     TokenAccount::unpack(&pda_bstocklqdy_token_account.data.borrow())?;

        let amm_ratio = pda_busd_token_account_info.amount / pda_bstock_token_account_info.amount;
        let bstock_amount = busd_amount * amm_ratio;

        let (pda, bump_seed) = Pubkey::find_program_address(&[b"bravv"], program_id);

        let amm_account = next_account_info(account_info_iter)?;
        let amm_info = AMM::unpack(&amm_account.data.borrow())?;
        if amm_info.busd_token_account_pubkey != *pda_busd_token_account.key || amm_info.bstock_token_account_pubkey != *pda_bstock_token_account.key {
            return Err(ProgramError::InvalidAccountData);
        }
        let token_program = next_account_info(account_info_iter)?;
        let transfer_bstock_to_amm = spl_token::instruction::transfer(
            token_program.key,
            user_bstock_token_account.key,
            pda_bstock_token_account.key,
            user_account.key,
            &[&user_account.key],
            bstock_amount,
        )?;
        msg!("Calling the token program to transfer tokens to the AMM...");
        invoke(
            &transfer_bstock_to_amm,
            &[
                user_bstock_token_account.clone(),
                pda_bstock_token_account.clone(),
                user_account.clone(),
                token_program.clone(),
            ],
        )?;

        let pda_account = next_account_info(account_info_iter)?;

        let transfer_busd_to_user = spl_token::instruction::transfer(
            token_program.key,
            pda_busd_token_account.key,
            user_busd_token_account.key,
            &pda,
            &[&pda],
            bstock_amount,
        )?;
        msg!("Calling the token program to transfer bStock to the user...");
        invoke_signed(
            &transfer_busd_to_user,
            &[
                pda_bstock_token_account.clone(),
                user_bstock_token_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"bravv"[..], &[bump_seed]]],
        )?;

        Ok(())
    }

    fn process_close_amm(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer_account = next_account_info(account_info_iter)?;
        let pdas_busd_token_account = next_account_info(account_info_iter)?;
        let pdas_bstock_token_account = next_account_info(account_info_iter)?;
        let pdas_bstocklqdy_token_account = next_account_info(account_info_iter)?;
        
        let amm_account = next_account_info(account_info_iter)?;
        let amm_info = AMM::unpack(&amm_account.data.borrow())?;
        if amm_info.initializer_account_pubkey != *initializer_account.key {
            return Err(AMMError::InvalidInitializer.into());
        }

        let token_program = next_account_info(account_info_iter)?;
        let pda_account = next_account_info(account_info_iter)?;

        let (pda, bump_seed) = Pubkey::find_program_address(&[b"bravv"], program_id);

        let close_pdas_busd_account = spl_token::instruction::close_account(
            token_program.key,
            pdas_busd_token_account.key,
            initializer_account.key,
            &pda,
            &[&pda],
        )?;
        let close_pdas_bstock_account = spl_token::instruction::close_account(
            token_program.key,
            pdas_busd_token_account.key,
            initializer_account.key,
            &pda,
            &[&pda],
        )?;
        let close_pdas_bstocklqdy_account = spl_token::instruction::close_account(
            token_program.key,
            pdas_busd_token_account.key,
            initializer_account.key,
            &pda,
            &[&pda],
        )?;
        msg!("Calling the token program to close pda's accounts...");
        invoke_signed(
            &close_pdas_busd_account,
            &[
                pdas_busd_token_account.clone(),
                initializer_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[bump_seed]]],
        )?;
        invoke_signed(
            &close_pdas_bstock_account,
            &[
                pdas_bstock_token_account.clone(),
                initializer_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[bump_seed]]],
        )?;
        invoke_signed(
            &close_pdas_bstocklqdy_account,
            &[
                pdas_bstocklqdy_token_account.clone(),
                initializer_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"escrow"[..], &[bump_seed]]],
        )?;

        msg!("Closing the escrow account...");
        **initializer_account.lamports.borrow_mut() = initializer_account
            .lamports()
            .checked_add(pda_account.lamports())
            .ok_or(AMMError::AmountOverflow)?;
        **pda_account.lamports.borrow_mut() = 0;
        *pda_account.data.borrow_mut() = &mut [];

        Ok(())
    }

}
