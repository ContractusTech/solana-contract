use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;
use std::convert::Into;
use std::str::FromStr;

// NEED CHECK: CTUS address
// Devnet: CyhjLfsfDz7rtszqBGaHiFrBbck2LNKEXQkywqNrGVyw
// Mainnet: ---
// Localnet: 67sJHNFLxkREsdu35n8tmfudv1ZU59XhBYjc6rivMt2V
static SERVICE_TOKEN_ADDRESS_MINT: &'static str = "67sJHNFLxkREsdu35n8tmfudv1ZU59XhBYjc6rivMt2V";

// NEED CHECK: Admin level
// Devnet: 3aDaxu2XwsGmj7amUnrxaHoKTtKJUqebkYP9HJTkP434
// Mainnet: ---
// Localnet: TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
static SERVICE_ACCOUNT_ADDRESS: &'static str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"; 

// NEED CHECK: 10_000 CTUS
static HOLDER_MODE_AMOUNT: u64 = 10000000000000; 

declare_id!("9kpWdyR2qtNT21MhLRTBbT21v5thz9hhB3zaPUhr6tbE");

#[program]
pub mod deal_contract {
    use super::*;

    const AUTHORITY_SEED: &[u8] = b"auth";
    
    pub fn initialize_with_checker(
        _ctx: Context<Initialize>,
        _id: Vec<u8>,
        amount: u64,
        service_fee: u64,
        checker_fee: u64,
        holder_mode: bool
    ) -> Result<()> {

        if _ctx.accounts.deal_state.is_started {
            return Err(ErrorCode::AlreadyStarted.into());
        }

        if amount == 0 {
            return Err(ErrorCode::AmountTooLow.into());
        }

        let service_token_mint: Pubkey = Pubkey::from_str(SERVICE_TOKEN_ADDRESS_MINT).unwrap();

        if holder_mode
        && _ctx.accounts.client_service_token_account.amount < HOLDER_MODE_AMOUNT 
        && _ctx.accounts.client_service_token_account.mint != service_token_mint 
        && _ctx.accounts.client_token_account.mint != service_token_mint {
            return Err(ErrorCode::HolderModeUnavailable.into());
        }
        if !holder_mode && service_fee == 0 {
            return Err(ErrorCode::FeeIsTooLow.into());
        }

        _ctx.accounts.deal_state.is_started = true;

        _ctx.accounts.deal_state.client_key = *_ctx.accounts.client.key;
        _ctx.accounts.deal_state.executor_key = *_ctx.accounts.executor.to_account_info().key;
        _ctx.accounts.deal_state.checker_key = *_ctx.accounts.checker.to_account_info().key;
        _ctx.accounts.deal_state.deposit_key = *_ctx.accounts.deposit_account.to_account_info().key;
        _ctx.accounts.deal_state.authority_key = *_ctx.accounts.authority.to_account_info().key;
        
        _ctx.accounts.deal_state.holder_mode_deposit_key =  *_ctx.accounts.holder_deposit_account.to_account_info().key;
        
        _ctx.accounts.deal_state.client_token_account_key = *_ctx.accounts.client_token_account.to_account_info().key;
        _ctx.accounts.deal_state.executor_token_account_key = *_ctx.accounts.executor_token_account.to_account_info().key;
        _ctx.accounts.deal_state.checker_token_account_key = *_ctx.accounts.checker_token_account.to_account_info().key;
        // ctx.accounts.deal_state.service_key = Pubkey::from_str(SERVICE_ACCOUNT_ADDRESS).unwrap();

        _ctx.accounts.deal_state.bump = *_ctx.bumps.get("deal_state").unwrap();
        _ctx.accounts.deal_state.deposit_bump = *_ctx.bumps.get("deposit_account").unwrap();
        _ctx.accounts.deal_state.authority_bump = *_ctx.bumps.get("authority").unwrap();
        _ctx.accounts.deal_state.holder_deposit_bump = *_ctx.bumps.get("holder_deposit_account").unwrap();
        
        _ctx.accounts.deal_state.with_bond = false;
        _ctx.accounts.deal_state.checker_fee = checker_fee;
        _ctx.accounts.deal_state.amount = amount;

        token::set_authority(
            _ctx.accounts.into_set_authority_context(),
            AuthorityType::AccountOwner,
            Some(*_ctx.accounts.authority.to_account_info().key),
        )?;

        token::set_authority(
            _ctx.accounts.into_set_authority_holder_context(),
            AuthorityType::AccountOwner,
            Some(*_ctx.accounts.authority.to_account_info().key),
        )?;

        token::transfer(
            _ctx.accounts.into_transfer_to_pda_context(),
            amount + checker_fee,
        )?;

        if holder_mode {
            token::transfer(
                _ctx.accounts.into_transfer_to_holder_mode_account(),
                HOLDER_MODE_AMOUNT,
            )?;
        } else if service_fee > 0 {
            token::transfer(
                _ctx.accounts.into_transfer_to_service_account(),
                service_fee,
            )?;
        }

        Ok(())
    }

    pub fn initialize_with_bond(
        _ctx: Context<InitializeBond>,
        _id: Vec<u8>,
        amount: u64,
        client_bond_amount: u64,
        executor_bond_amount: u64,
        service_fee: u64,
        deadline_ts: i64,
        holder_mode: bool
    ) -> Result<()> {

        if _ctx.accounts.deal_state.is_started {
            return Err(ErrorCode::AlreadyStarted.into());
        }

        let clock = Clock::get()?;
        let current_ts = clock.unix_timestamp;
        if deadline_ts < current_ts {
            return Err(ErrorCode::DeadlineExpired.into());
        }

        if amount == 0 {
            return Err(ErrorCode::AmountTooLow.into());
        }

        let service_token_mint: Pubkey = Pubkey::from_str(SERVICE_TOKEN_ADDRESS_MINT).unwrap();

        if holder_mode
        && _ctx.accounts.client_service_token_account.amount < HOLDER_MODE_AMOUNT 
        && _ctx.accounts.client_service_token_account.mint != service_token_mint 
        && _ctx.accounts.client_token_account.mint != service_token_mint {
            return Err(ErrorCode::HolderModeUnavailable.into());
        }
        if !holder_mode && service_fee == 0 {
            return Err(ErrorCode::FeeIsTooLow.into());
        }

        _ctx.accounts.deal_state.is_started = true;

        _ctx.accounts.deal_state.client_key = *_ctx.accounts.client.key;
        _ctx.accounts.deal_state.executor_key = *_ctx.accounts.executor.to_account_info().key;
        _ctx.accounts.deal_state.deposit_key = *_ctx.accounts.deposit_account.to_account_info().key;
        _ctx.accounts.deal_state.authority_key = *_ctx.accounts.authority.to_account_info().key;
        
        _ctx.accounts.deal_state.client_token_account_key = *_ctx.accounts.client_token_account.to_account_info().key;
        _ctx.accounts.deal_state.client_bond_deposit_key = *_ctx.accounts.deposit_client_bond_account.to_account_info().key;
        _ctx.accounts.deal_state.client_bond_token_account_key = *_ctx.accounts.client_bond_account.to_account_info().key;

        _ctx.accounts.deal_state.executor_token_account_key = *_ctx.accounts.executor_token_account.to_account_info().key;
        _ctx.accounts.deal_state.executor_bond_deposit_key = *_ctx.accounts.deposit_executor_bond_account.to_account_info().key;
        _ctx.accounts.deal_state.executor_bond_token_account_key = *_ctx.accounts.executor_bond_account.to_account_info().key;
        
        _ctx.accounts.deal_state.holder_mode_deposit_key =  *_ctx.accounts.holder_deposit_account.to_account_info().key;
        
        // ctx.accounts.deal_state.service_key = Pubkey::from_str(SERVICE_ACCOUNT_ADDRESS).unwrap();

        _ctx.accounts.deal_state.bump = *_ctx.bumps.get("deal_state").unwrap();
        _ctx.accounts.deal_state.deposit_bump = *_ctx.bumps.get("deposit_account").unwrap();
        _ctx.accounts.deal_state.authority_bump = *_ctx.bumps.get("authority").unwrap();
        _ctx.accounts.deal_state.holder_deposit_bump = *_ctx.bumps.get("holder_deposit_account").unwrap();
        _ctx.accounts.deal_state.client_bond_deposit_bump = *_ctx.bumps.get("deposit_client_bond_account").unwrap();
        _ctx.accounts.deal_state.executor_bond_deposit_bump = *_ctx.bumps.get("deposit_executor_bond_account").unwrap();

        _ctx.accounts.deal_state.amount = amount;
        _ctx.accounts.deal_state.client_bond_amount = client_bond_amount;
        _ctx.accounts.deal_state.executor_bond_amount = executor_bond_amount;

        _ctx.accounts.deal_state.with_bond = true;
        _ctx.accounts.deal_state.deadline_ts = deadline_ts;
        
        token::set_authority(
            _ctx.accounts.into_set_authority_context(),
            AuthorityType::AccountOwner,
            Some(*_ctx.accounts.authority.to_account_info().key),
        )?;

        token::set_authority(
            _ctx.accounts.into_set_authority_holder_context(),
            AuthorityType::AccountOwner,
            Some(*_ctx.accounts.authority.to_account_info().key),
        )?;

        token::transfer(
            _ctx.accounts.into_transfer_to_pda_context(),
            amount,
        )?;

        if holder_mode {
            token::transfer(
                _ctx.accounts.into_transfer_to_holder_mode_account(),
                HOLDER_MODE_AMOUNT,
            )?;
        } else if service_fee > 0 {
            token::transfer(
                _ctx.accounts.into_transfer_to_service_account(),
                service_fee,
            )?;
        }

        if client_bond_amount > 0 {
            token::set_authority(
                _ctx.accounts.into_set_authority_client_bond_context(),
                AuthorityType::AccountOwner,
                Some(*_ctx.accounts.authority.to_account_info().key),
            )?;

            token::transfer(
                _ctx.accounts.into_transfer_to_client_bond_account(),
                client_bond_amount,
            )?;
        }

        if executor_bond_amount > 0 {
            token::set_authority(
                _ctx.accounts.into_set_authority_executor_bond_context(),
                AuthorityType::AccountOwner,
                Some(*_ctx.accounts.authority.to_account_info().key),
            )?;

            token::transfer(
                _ctx.accounts.into_transfer_to_executor_bond_account(),
                executor_bond_amount,
            )?;
        }

        Ok(())
    }

    pub fn finish(
        _ctx: Context<Finish>, 
        _id: Vec<u8>
    ) -> Result<()> {

        if !_ctx.accounts.deal_state.is_started {
            return Err(ErrorCode::NotStarted.into());
        }
        let seeds = &[
            &_id, 
            &AUTHORITY_SEED[..],
            _ctx.accounts.deal_state.client_key.as_ref(),
            _ctx.accounts.deal_state.executor_key.as_ref(),
            &[_ctx.accounts.deal_state.authority_bump]];

        token::transfer(
            _ctx.accounts
                .into_transfer_to_executor_token_account_context()
                .with_signer(&[&seeds[..]]),
                _ctx.accounts.deal_state.amount
        )?;
        
        if _ctx.accounts.deal_state.checker_fee > 0 {
            token::transfer(
                _ctx.accounts
                    .into_transfer_to_checker_token_account_context()
                    .with_signer(&[&seeds[..]]),
                    _ctx.accounts.deal_state.checker_fee
            )?;
        } 

        token::close_account(
            _ctx.accounts
                .into_close_context()
                .with_signer(&[&seeds[..]])
        )?;

        Ok(())
    }

    pub fn finish_with_bond(
        _ctx: Context<FinishWithBond>, 
        _id: Vec<u8>
    ) -> Result<()> {

        if !_ctx.accounts.deal_state.is_started {
            return Err(ErrorCode::NotStarted.into());
        }
        let seeds = &[
            &_id, 
            &AUTHORITY_SEED[..],
            _ctx.accounts.deal_state.client_key.as_ref(),
            _ctx.accounts.deal_state.executor_key.as_ref(),
            &[_ctx.accounts.deal_state.authority_bump]];

        token::transfer(
            _ctx.accounts
                .into_transfer_to_executor_token_account_context()
                .with_signer(&[&seeds[..]]),
                _ctx.accounts.deal_state.amount
        )?;
        
        if _ctx.accounts.deal_state.checker_fee > 0 {
            token::transfer(
                _ctx.accounts
                    .into_transfer_to_checker_token_account_context()
                    .with_signer(&[&seeds[..]]),
                    _ctx.accounts.deal_state.checker_fee
            )?;
        } 

        if _ctx.accounts.deal_state.client_bond_amount > 0 {
            token::transfer(
                _ctx.accounts
                    .into_transfer_to_bond_client_token_account_context()
                    .with_signer(&[&seeds[..]]),
                    _ctx.accounts.deal_state.client_bond_amount
            )?;
        }

        if _ctx.accounts.deal_state.executor_bond_amount > 0 {
            token::transfer(
                _ctx.accounts
                    .into_transfer_to_bond_executor_token_account_context()
                    .with_signer(&[&seeds[..]]),
                    _ctx.accounts.deal_state.executor_bond_amount
            )?;
        }

        token::close_account(
            _ctx.accounts
                .into_close_context()
                .with_signer(&[&seeds[..]])
        )?;

        Ok(())
    }

    pub fn cancel(
        _ctx: Context<Cancel>,
        _id: Vec<u8>
    ) -> Result<()> {

        if !_ctx.accounts.deal_state.is_started {
            return Err(ErrorCode::NotStarted.into());
        }

        if _ctx.accounts.deal_state.with_bond {
            return Err(ErrorCode::NeedCancelWithBond.into());
        }

        if _ctx.accounts.deal_state.deadline_ts > 0 {
            let clock = Clock::get()?;
            let current_ts = clock.unix_timestamp;
            if current_ts > _ctx.accounts.deal_state.deadline_ts {
                return Err(ErrorCode::DeadlineNotCome.into());
            }
        }

        let seeds = &[
            &_id, 
            &AUTHORITY_SEED[..],
            _ctx.accounts.deal_state.client_key.as_ref(),
            _ctx.accounts.deal_state.executor_key.as_ref(),
            &[_ctx.accounts.deal_state.authority_bump]];


        let amount = _ctx.accounts.deal_state.amount + _ctx.accounts.deal_state.checker_fee;
        
        token::transfer(
            _ctx.accounts
                .into_transfer_to_client_token_account_context()
                .with_signer(&[&seeds[..]]),
                amount
        )?;

        token::close_account(
            _ctx.accounts
                .into_close_context()
                .with_signer(&[&seeds[..]])
        )?;

        Ok(())
    }

    pub fn cancel_with_bond(
        _ctx: Context<CancelWithBond>,
        _id: Vec<u8>
    ) -> Result<()> {

        if !_ctx.accounts.deal_state.is_started {
            return Err(ErrorCode::NotStarted.into());
        }

        if !_ctx.accounts.deal_state.with_bond {
            return Err(ErrorCode::NeedCancelWithoutBond.into());
        }

        if _ctx.accounts.deal_state.deadline_ts > 0 {
            let clock = Clock::get()?;
            let current_ts = clock.unix_timestamp;
            if _ctx.accounts.deal_state.deadline_ts > current_ts  {
                return Err(ErrorCode::DeadlineNotCome.into());
            }
        }

        let seeds = &[
            &_id, 
            &AUTHORITY_SEED[..],
            _ctx.accounts.deal_state.client_key.as_ref(),
            _ctx.accounts.deal_state.executor_key.as_ref(),
            &[_ctx.accounts.deal_state.authority_bump]];


        let amount = _ctx.accounts.deal_state.amount + _ctx.accounts.deal_state.checker_fee;
        
        token::transfer(
            _ctx.accounts
                .into_transfer_to_client_token_account_context()
                .with_signer(&[&seeds[..]]),
                amount
        )?;

        if _ctx.accounts.deal_state.client_bond_amount > 0 {
            token::transfer(
                _ctx.accounts
                    .into_transfer_to_bond_client_token_account_context()
                    .with_signer(&[&seeds[..]]),
                    _ctx.accounts.deal_state.client_bond_amount
            )?;
        }

        if _ctx.accounts.deal_state.executor_bond_amount > 0 {
            token::transfer(
                _ctx.accounts
                    .into_transfer_to_bond_executor_token_account_context()
                    .with_signer(&[&seeds[..]]),
                    _ctx.accounts.deal_state.executor_bond_amount
            )?;
        }

        token::close_account(
            _ctx.accounts
                .into_close_context()
                .with_signer(&[&seeds[..]])
        )?;

        Ok(())
    }

    pub fn update_checker(
        _ctx: Context<UpdateChecker>,
        _id: Vec<u8>
    ) -> Result<()> {
        // TODO: - 
        return Err(ErrorCode::NotImplemented.into());
    }
}

#[derive(Accounts)]
#[instruction(id: Vec<u8>, amount: u64, service_fee: u64, checker_fee: u64, holder_mode: bool)]
pub struct Initialize<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut, 
        signer, 
        constraint = *executor.to_account_info().key != *client.to_account_info().key &&  *checker.to_account_info().key != *client.to_account_info().key
    )]
    pub client: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut,signer)]
    pub executor: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub checker: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub payer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = service_fee_account.mint.key() == client_token_account.mint.key()
    )]
    pub service_fee_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = executor_token_account.mint.key() == client_token_account.mint.key(),
        constraint = executor_token_account.owner.key() == executor.key()
    )]
    pub executor_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = client_token_account.owner.key() == client.key(),
        constraint = executor_token_account.mint.key() == mint.key(),
        constraint = client_token_account.amount >= (amount + service_fee + checker_fee),
        constraint = (holder_mode && client_token_account.mint.key() == Pubkey::from_str(SERVICE_TOKEN_ADDRESS_MINT).unwrap() && client_token_account.amount > (amount + service_fee + checker_fee + HOLDER_MODE_AMOUNT)) 
                        || !holder_mode
    )]
    pub client_token_account: Account<'info, TokenAccount>,
    #[account(
        constraint = client_service_token_account.owner.key() == *client.to_account_info().key,
        constraint = client_service_token_account.mint.key() == Pubkey::from_str(SERVICE_TOKEN_ADDRESS_MINT).unwrap(),
    )]
    pub client_service_token_account: Box<Account<'info, TokenAccount>>,
     /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = checker_token_account.mint.key() == mint.key(),
        constraint = checker_token_account.owner.key() == checker.key()
    )]
    pub checker_token_account: Box<Account<'info, TokenAccount>>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        seeds = [&id, b"deposit".as_ref(), client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
        payer = payer,
        token::mint = mint,
        token::authority = payer,
    )]
    pub deposit_account: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        seeds = [&id, b"holder_deposit".as_ref(), client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
        payer = payer,
        token::mint = holder_mint,
        token::authority = payer,
    )]
    pub holder_deposit_account: Box<Account<'info, TokenAccount>>,
    pub holder_mint: Box<Account<'info, Mint>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        seeds = [&id, b"auth".as_ref(), client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
    )]
    pub authority: AccountInfo<'info>,
   
    #[account(
        init_if_needed,
        seeds = [&id, b"state".as_ref(), client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
        payer = payer, 
        space = DealState::space()
    )]
    pub deal_state: Box<Account<'info, DealState>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>
}

#[derive(Accounts)]
#[instruction(id: Vec<u8>, amount: u64, client_bond_amount: u64, executor_bond_amount: u64, service_fee: u64, deadline_ts: i64, holder_mode: bool)]
pub struct InitializeBond<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut, 
        signer, 
        constraint = *executor.to_account_info().key != *client.to_account_info().key
    )]
    pub client: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub executor: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub payer: AccountInfo<'info>,
    #[account(
        mut,
        constraint = service_fee_account.mint.key() == client_token_account.mint.key()
    )]
    pub service_fee_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = executor_token_account.mint.key() == client_token_account.mint.key(),
        constraint = executor_token_account.owner.key() == executor.key()
    )]
    pub executor_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = client_token_account.owner.key() == client.key(),
        constraint = executor_token_account.mint.key() == mint.key(),
        constraint = client_token_account.amount >= (amount + service_fee),
        constraint = (holder_mode && client_token_account.mint.key() == client_service_token_account.mint.key() && client_token_account.amount > (amount + HOLDER_MODE_AMOUNT)) 
                        || client_token_account.mint.key() != client_service_token_account.mint.key()
    )]
    pub client_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        constraint = client_service_token_account.owner.key() == *client.to_account_info().key,
        constraint = client_service_token_account.mint.key() == Pubkey::from_str(SERVICE_TOKEN_ADDRESS_MINT).unwrap(),
        constraint = (holder_mode && client_service_token_account.amount >= HOLDER_MODE_AMOUNT) || !holder_mode
    )]
    pub client_service_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = client_bond_account.owner.key() == *client.to_account_info().key,
        constraint = client_bond_account.mint.key() == client_bond_mint.key(),
        constraint = (client_bond_account.mint == mint.key() && client_bond_account.amount >= (client_bond_amount + service_fee + amount))
            || (client_bond_account.mint.key() != mint.key() && client_bond_account.amount >= client_bond_amount)
    )]
    pub client_bond_account: Box<Account<'info, TokenAccount>>,
    pub client_bond_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        constraint = executor_bond_account.owner.key() == *executor.to_account_info().key,
        constraint = executor_bond_account.mint.key() == executor_bond_mint.key(),
        constraint = executor_bond_account.amount >= executor_bond_amount
    )]
    pub executor_bond_account: Box<Account<'info, TokenAccount>>,
    pub executor_bond_mint: Box<Account<'info, Mint>>,

    pub mint: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        seeds = [&id, b"deposit".as_ref(), client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
        payer = payer,
        token::mint = mint,
        token::authority = payer,
    )]
    pub deposit_account: Box<Account<'info, TokenAccount>>,
     /// CHECK: This is not dangerous because we don't read or write from this account
     #[account(
        seeds = [&id, b"auth".as_ref(), client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
    )]
    pub authority: AccountInfo<'info>,
    #[account(
        init_if_needed,
        seeds = [&id, b"deposit_bond_client".as_ref(), client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
        payer = payer,
        token::mint = client_bond_mint,
        token::authority = payer,
    )]
    pub deposit_client_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        seeds = [&id, b"deposit_bond_executor".as_ref(), client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
        payer = payer,
        token::mint = executor_bond_mint,
        token::authority = payer,
    )]
    pub deposit_executor_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        seeds = [&id, b"holder_deposit".as_ref(), client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
        payer = payer,
        token::mint = holder_mint,
        token::authority = payer,
    )]
    pub holder_deposit_account: Box<Account<'info, TokenAccount>>,
    pub holder_mint: Box<Account<'info, Mint>>,
   
    #[account(
        init_if_needed,
        seeds = [&id, b"state".as_ref(), client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
        payer = payer, 
        space = DealState::space()
    )]
    pub deal_state: Box<Account<'info, DealState>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>
}

#[derive(Accounts)]
#[instruction(id: Vec<u8>)]
pub struct Cancel<'info> {    
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub initializer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        constraint = *authority.to_account_info().key == deal_state.authority_key
    )]
    pub authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = deal_state.deposit_key == *deposit_account.to_account_info().key
    )]
    pub deposit_account: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = *client_token_account.to_account_info().key == deal_state.client_token_account_key
    )]
    pub client_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [&id, b"state".as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.bump,
        constraint = (*initializer.to_account_info().key == deal_state.client_key || *initializer.to_account_info().key == deal_state.executor_key || *initializer.to_account_info().key == deal_state.checker_key || *initializer.to_account_info().key == Pubkey::from_str(SERVICE_ACCOUNT_ADDRESS).unwrap()),
        close = initializer
    )]
    pub deal_state: Box<Account<'info, DealState>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>
}

#[derive(Accounts)]
#[instruction(id: Vec<u8>)]
pub struct CancelWithBond<'info> {    
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub initializer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        constraint = *authority.to_account_info().key == deal_state.authority_key
    )]
    pub authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = *deposit_account.to_account_info().key == deal_state.deposit_key
    )]
    pub deposit_account: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = *client_token_account.to_account_info().key == deal_state.client_token_account_key
    )]
    pub client_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = *client_bond_account.to_account_info().key == deal_state.client_bond_token_account_key
    )]
    pub client_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = *executor_bond_account.to_account_info().key == deal_state.executor_bond_token_account_key
    )]
    pub executor_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = *deposit_client_bond_account.to_account_info().key == deal_state.client_bond_deposit_key
    )]
    pub deposit_client_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = *deposit_executor_bond_account.to_account_info().key == deal_state.executor_bond_deposit_key
    )]
    pub deposit_executor_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [&id, b"state".as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.bump,
        constraint = (*initializer.to_account_info().key == deal_state.client_key || *initializer.to_account_info().key == deal_state.executor_key || *initializer.to_account_info().key == deal_state.checker_key || *initializer.to_account_info().key == Pubkey::from_str(SERVICE_ACCOUNT_ADDRESS).unwrap()),
        close = initializer
    )]
    pub deal_state: Box<Account<'info, DealState>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>
}

#[derive(Accounts)]
#[instruction(id: Vec<u8>)]
pub struct Finish<'info> { 
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut, 
        signer,)]
    pub initializer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        constraint = *authority.to_account_info().key == deal_state.authority_key
    )]
    pub authority: AccountInfo<'info>,
    #[account(
        mut,
        constraint = deal_state.deposit_key == *deposit_account.to_account_info().key,
    )]
    pub deposit_account: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        constraint = deal_state.holder_mode_deposit_key == *holder_deposit_account.to_account_info().key,
        mut,
    )]
    pub holder_deposit_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = deal_state.executor_token_account_key == *executor_token_account.to_account_info().key,
    )]
    pub executor_token_account: Account<'info, TokenAccount>,
     /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = deal_state.checker_token_account_key == *checker_token_account.to_account_info().key,
    )]
    pub checker_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [&id, b"state".as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.bump,
        constraint = (*initializer.to_account_info().key == deal_state.client_key || *initializer.to_account_info().key == deal_state.checker_key),
        constraint = deal_state.deposit_key == *deposit_account.to_account_info().key,
        constraint = *executor_token_account.to_account_info().key == deal_state.executor_token_account_key,
        constraint = *checker_token_account.to_account_info().key == deal_state.checker_token_account_key,
        close = initializer
    )]
    pub deal_state: Box<Account<'info, DealState>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>
}

#[derive(Accounts)]
#[instruction(id: Vec<u8>)]
pub struct FinishWithBond<'info> { 
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut, 
        signer)]
    pub initializer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        constraint = *authority.to_account_info().key == deal_state.authority_key
    )]
    pub authority: AccountInfo<'info>,
    #[account(
        mut,
        constraint = deal_state.deposit_key == *deposit_account.to_account_info().key
    )]
    pub deposit_account: Account<'info, TokenAccount>,
    #[account(
        constraint = deal_state.holder_mode_deposit_key == *holder_deposit_account.to_account_info().key
    )]
    pub holder_deposit_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = deal_state.executor_token_account_key == *executor_token_account.to_account_info().key,
    )]
    pub executor_token_account: Account<'info, TokenAccount>,
     /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = deal_state.checker_token_account_key == *checker_token_account.to_account_info().key,
    )]
    pub checker_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = client_bond_account.owner.key() == deal_state.client_bond_token_account_key
    )]
    pub client_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = executor_bond_account.owner.key() == deal_state.executor_bond_token_account_key
    )]
    pub executor_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = *deposit_client_bond_account.to_account_info().key == deal_state.client_bond_deposit_key
    )]
    pub deposit_client_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = *deposit_client_bond_account.to_account_info().key == deal_state.executor_bond_deposit_key
    )]
    pub deposit_executor_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [&id, b"state".as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.bump,
        constraint = (*initializer.to_account_info().key == deal_state.client_key || *initializer.to_account_info().key == deal_state.checker_key),
        constraint = deal_state.deposit_key == *deposit_account.to_account_info().key,
        constraint = *executor_token_account.to_account_info().key == deal_state.executor_token_account_key,
        constraint = *checker_token_account.to_account_info().key == deal_state.checker_token_account_key,
        close = initializer
    )]
    pub deal_state: Box<Account<'info, DealState>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>
}

#[derive(Accounts)]
#[instruction(id: Vec<u8>)]
pub struct UpdateChecker<'info> { 
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut, 
        signer,
        constraint = Pubkey::from_str(SERVICE_ACCOUNT_ADDRESS).unwrap() == *initializer.to_account_info().key,
    )]
    pub initializer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account( mut)]
    pub client: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub executor: AccountInfo<'info>,
   
    #[account(
        mut,
        seeds = [&id, b"state".as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.bump
    )]
    pub deal_state: Box<Account<'info, DealState>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>
}

#[account]
pub struct DealState {
    pub is_started: bool,
    pub with_bond: bool,

    pub client_key: Pubkey,
    pub client_token_account_key: Pubkey,
    pub client_bond_token_account_key: Pubkey,
    pub client_bond_deposit_key: Pubkey,

    pub executor_key: Pubkey,
    pub executor_token_account_key: Pubkey,
    pub executor_bond_token_account_key: Pubkey,
    pub executor_bond_deposit_key: Pubkey,

    pub checker_key: Pubkey,
    pub checker_token_account_key: Pubkey,
    pub deposit_key: Pubkey,
    pub holder_mode_deposit_key: Pubkey,

    pub authority_key: Pubkey,
    // pub service_key: Pubkey,

    pub amount: u64,
    pub client_bond_amount: u64,
    pub executor_bond_amount: u64,
    pub checker_fee: u64,
    
    pub deadline_ts: i64,

    pub bump: u8,
    pub deposit_bump: u8,
    pub authority_bump: u8,
    pub holder_deposit_bump: u8,
    pub client_bond_deposit_bump: u8,
    pub executor_bond_deposit_bump: u8,
}

impl DealState {
    pub fn space() -> usize {
        2 + (14 * 32) +  (5 * 8) + 6 + 8
    }
}

// Initialize
impl<'info> Initialize<'info> {
    fn into_transfer_to_pda_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self
                .client_token_account
                .to_account_info(),
            to: self.deposit_account.to_account_info(),
            authority: self.client.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_service_account(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self
                .client_token_account
                .to_account_info(),
            to: self.service_fee_account.to_account_info(),
            authority: self.client.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }


    fn into_transfer_to_holder_mode_account(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self
                .client_token_account
                .to_account_info(),
            to: self.holder_deposit_account.to_account_info(),
            authority: self.client.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.deposit_account.to_account_info(),
            current_authority: self.payer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_set_authority_holder_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.holder_deposit_account.to_account_info(),
            current_authority: self.payer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

// Initialize Bond
impl<'info> InitializeBond<'info> {
    fn into_transfer_to_pda_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self
                .client_token_account
                .to_account_info(),
            to: self.deposit_account.to_account_info(),
            authority: self.client.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_service_account(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self
                .client_token_account
                .to_account_info(),
            to: self.service_fee_account.to_account_info(),
            authority: self.client.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_holder_mode_account(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self
                .client_token_account
                .to_account_info(),
            to: self.holder_deposit_account.to_account_info(),
            authority: self.client.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_executor_bond_account(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self
                .executor_bond_account
                .to_account_info(),
            to: self.deposit_executor_bond_account.to_account_info(),
            authority: self.executor.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_client_bond_account(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self
                .client_bond_account.to_account_info(),
            to: self.deposit_client_bond_account.to_account_info(),
            authority: self.client.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.deposit_account.to_account_info(),
            current_authority: self.payer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_set_authority_holder_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.holder_deposit_account.to_account_info(),
            current_authority: self.payer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_set_authority_executor_bond_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.deposit_executor_bond_account.to_account_info(),
            current_authority: self.payer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_set_authority_client_bond_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.deposit_client_bond_account.to_account_info(),
            current_authority: self.payer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

// Cancel
impl<'info> Cancel<'info> {

    fn into_transfer_to_client_token_account_context(
        &self
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.deposit_account.to_account_info(),
            to: self.client_token_account.to_account_info(),
            authority: self.authority.clone()
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.deposit_account.to_account_info(),
            destination: self.initializer.to_account_info(),
            authority: self.authority.clone()
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

// Cancel With Bond
impl<'info> CancelWithBond<'info> {

    fn into_transfer_to_client_token_account_context(
        &self
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.deposit_account.to_account_info(),
            to: self.client_token_account.to_account_info(),
            authority: self.authority.clone()
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_bond_client_token_account_context(
        &self
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.deposit_client_bond_account.to_account_info(),
            to: self.client_bond_account.to_account_info(),
            authority: self.authority.clone()
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_bond_executor_token_account_context(
        &self
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.deposit_executor_bond_account.to_account_info(),
            to: self.executor_bond_account.to_account_info(),
            authority: self.authority.clone()
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.deposit_account.to_account_info(),
            destination: self.initializer.to_account_info(),
            authority: self.authority.clone()
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

// Finish
impl<'info> Finish<'info> {

    fn into_transfer_to_executor_token_account_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.deposit_account.to_account_info(),
            to: self.executor_token_account.to_account_info(),
            authority: self.authority.clone()
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_checker_token_account_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.deposit_account.to_account_info(),
            to: self.checker_token_account.to_account_info(),
            authority: self.authority.clone()
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.deposit_account.to_account_info(),
            destination: self.initializer.to_account_info(),
            authority: self.authority.clone()
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

// Finish with Bond
impl<'info> FinishWithBond<'info> {

    fn into_transfer_to_executor_token_account_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.deposit_account.to_account_info(),
            to: self.executor_token_account.to_account_info(),
            authority: self.authority.clone()
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_checker_token_account_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.deposit_account.to_account_info(),
            to: self.checker_token_account.to_account_info(),
            authority: self.authority.clone()
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_bond_client_token_account_context(
        &self
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.deposit_client_bond_account.to_account_info(),
            to: self.client_bond_account.to_account_info(),
            authority: self.authority.clone()
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_bond_executor_token_account_context(
        &self
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.deposit_executor_bond_account.to_account_info(),
            to: self.executor_bond_account.to_account_info(),
            authority: self.authority.clone()
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        let cpi_accounts = CloseAccount {
            account: self.deposit_account.to_account_info(),
            destination: self.initializer.to_account_info(),
            authority: self.authority.clone()
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("The deal already started")]
    AlreadyStarted,
    #[msg("Th deal not started")]
    NotStarted,
    #[msg("The fee is too low")]
    FeeIsTooLow,
    #[msg("Holder mode unavailable")]
    HolderModeUnavailable,
    #[msg("The amount is too small.")]
    AmountTooLow,
    #[msg("The deadline has not yet come.")]
    DeadlineNotCome,
    #[msg("Deadline expired")]
    DeadlineExpired,
    #[msg("The deal need cancel with bond")]
    NeedCancelWithBond,
    #[msg("The deal need cancel without bond")]
    NeedCancelWithoutBond,
    #[msg("Not implemented method")]
    NotImplemented,
}