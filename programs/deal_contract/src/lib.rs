use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;
use std::convert::Into;
use std::str::FromStr;

static SERVICE_TOKEN_ADDRESS_MINT: &'static str = "67sJHNFLxkREsdu35n8tmfudv1ZU59XhBYjc6rivMt2V"; // NEED CHECK: CTUS address
static SERVICE_ACCOUNT_ADDRESS: &'static str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"; // NEED CHECK: Admin level
static HOLDER_MODE_AMOUNT: u64 = 10000000000000; // NEED CHECK: 10_000 CTUS

declare_id!("9kpWdyR2qtNT21MhLRTBbT21v5thz9hhB3zaPUhr6tbE");

#[program]
pub mod deal_contract {
    use super::*;

    const AUTHORITY_SEED: &[u8] = b"auth";
    const DEPOSIT_SEED: &[u8] = b"deposit";
    
    pub fn initialize_with_checker(
        ctx: Context<Initialize>,
        id: Vec<u8>,
        amount: u64,
        service_fee: u64,
        checker_fee: u64,
        holder_mode: bool
    ) -> Result<()> {

        if ctx.accounts.deal_state.is_started {
            return Err(ErrorCode::AlreadyStarted.into());
        }

        if amount == 0 {
            return Err(ErrorCode::AmountTooLow.into());
        }

        let service_token_mint: Pubkey = Pubkey::from_str(SERVICE_TOKEN_ADDRESS_MINT).unwrap();

        if holder_mode
        && ctx.accounts.client_service_token_account.amount < HOLDER_MODE_AMOUNT 
        && ctx.accounts.client_service_token_account.mint != service_token_mint 
        && ctx.accounts.client_token_account.mint != service_token_mint {
            return Err(ErrorCode::HolderModeUnavailable.into());
        }
        if !holder_mode && service_fee == 0 {
            return Err(ErrorCode::FeeIsTooLow.into());
        }

        ctx.accounts.deal_state.is_started = true;

        ctx.accounts.deal_state.client_key = *ctx.accounts.client.key;
        ctx.accounts.deal_state.executor_key = *ctx.accounts.executor.to_account_info().key;
        ctx.accounts.deal_state.checker_key = *ctx.accounts.checker.to_account_info().key;
        ctx.accounts.deal_state.deposit_key = *ctx.accounts.deposit_account.to_account_info().key;
        ctx.accounts.deal_state.authority_key = *ctx.accounts.authority.to_account_info().key;
        
        ctx.accounts.deal_state.client_token_account_key = *ctx.accounts.client_token_account.to_account_info().key;
        ctx.accounts.deal_state.executor_token_account_key = *ctx.accounts.executor_token_account.to_account_info().key;
        ctx.accounts.deal_state.checker_token_account_key = *ctx.accounts.checker_token_account.to_account_info().key;
        ctx.accounts.deal_state.service_key = Pubkey::from_str(SERVICE_ACCOUNT_ADDRESS).unwrap();

        ctx.accounts.deal_state.bump = *ctx.bumps.get("deal_state").unwrap();
        ctx.accounts.deal_state.deposit_bump = *ctx.bumps.get("deposit_account").unwrap();
        ctx.accounts.deal_state.authority_bump = *ctx.bumps.get("authority").unwrap();
        ctx.accounts.deal_state.holder_deposit_bump = *ctx.bumps.get("holder_deposit_account").unwrap();
        
        
        ctx.accounts.deal_state.with_bond = false;
        ctx.accounts.deal_state.checker_fee = checker_fee;
        ctx.accounts.deal_state.amount = amount;

        token::set_authority(
            ctx.accounts.into_set_authority_context(),
            AuthorityType::AccountOwner,
            Some(*ctx.accounts.authority.to_account_info().key),
        )?;

        token::set_authority(
            ctx.accounts.into_set_authority_holder_context(),
            AuthorityType::AccountOwner,
            Some(*ctx.accounts.authority.to_account_info().key),
        )?;

        token::transfer(
            ctx.accounts.into_transfer_to_pda_context(),
            amount + checker_fee,
        )?;

        if holder_mode {
            token::transfer(
                ctx.accounts.into_transfer_to_holder_mode_account(),
                HOLDER_MODE_AMOUNT,
            )?;
        } else if service_fee > 0 {
            token::transfer(
                ctx.accounts.into_transfer_to_service_account(),
                service_fee,
            )?;
        }

        Ok(())
    }

    pub fn initialize_with_bond(
        ctx: Context<InitializeBond>,
        id: Vec<u8>,
        amount: u64,
        client_bond_amount: u64,
        executor_bond_amount: u64,
        service_fee: u64,
        deadline_ts: i64,
        holder_mode: bool
    ) -> Result<()> {

        if ctx.accounts.deal_state.is_started {
            return Err(ErrorCode::AlreadyStarted.into());
        }

        if amount == 0 {
            return Err(ErrorCode::AmountTooLow.into());
        }

        let service_token_mint: Pubkey = Pubkey::from_str(SERVICE_TOKEN_ADDRESS_MINT).unwrap();

        if holder_mode
        && ctx.accounts.client_service_token_account.amount < HOLDER_MODE_AMOUNT 
        && ctx.accounts.client_service_token_account.mint != service_token_mint 
        && ctx.accounts.client_token_account.mint != service_token_mint {
            return Err(ErrorCode::HolderModeUnavailable.into());
        }
        if !holder_mode && service_fee == 0 {
            return Err(ErrorCode::FeeIsTooLow.into());
        }

        ctx.accounts.deal_state.is_started = true;

        ctx.accounts.deal_state.client_key = *ctx.accounts.client.key;
        ctx.accounts.deal_state.executor_key = *ctx.accounts.executor.to_account_info().key;
        ctx.accounts.deal_state.deposit_key = *ctx.accounts.deposit_account.to_account_info().key;
        ctx.accounts.deal_state.authority_key = *ctx.accounts.authority.to_account_info().key;
        
        ctx.accounts.deal_state.client_token_account_key = *ctx.accounts.client_token_account.to_account_info().key;
        ctx.accounts.deal_state.executor_token_account_key = *ctx.accounts.executor_token_account.to_account_info().key;
        ctx.accounts.deal_state.executor_bond_deposit_key = *ctx.accounts.executor_bond_account.to_account_info().key;
        ctx.accounts.deal_state.client_bond_token_account_key = *ctx.accounts.client_bond_account.to_account_info().key;

        ctx.accounts.deal_state.service_key = Pubkey::from_str(SERVICE_ACCOUNT_ADDRESS).unwrap();

        ctx.accounts.deal_state.bump = *ctx.bumps.get("deal_state").unwrap();
        ctx.accounts.deal_state.deposit_bump = *ctx.bumps.get("deposit_account").unwrap();
        ctx.accounts.deal_state.authority_bump = *ctx.bumps.get("authority").unwrap();
        ctx.accounts.deal_state.holder_deposit_bump = *ctx.bumps.get("holder_deposit_account").unwrap();
        ctx.accounts.deal_state.client_bond_deposit_bump = *ctx.bumps.get("deposit_client_bond_account").unwrap();
        ctx.accounts.deal_state.executor_bond_deposit_bump = *ctx.bumps.get("deposit_executor_bond_account").unwrap();

        ctx.accounts.deal_state.amount = amount;
        ctx.accounts.deal_state.client_bond_amount = client_bond_amount;
        ctx.accounts.deal_state.executor_bond_amount = executor_bond_amount;

        ctx.accounts.deal_state.with_bond = true;
        ctx.accounts.deal_state.deadline_ts = deadline_ts;
        
        token::set_authority(
            ctx.accounts.into_set_authority_context(),
            AuthorityType::AccountOwner,
            Some(*ctx.accounts.authority.to_account_info().key),
        )?;

        token::set_authority(
            ctx.accounts.into_set_authority_holder_context(),
            AuthorityType::AccountOwner,
            Some(*ctx.accounts.authority.to_account_info().key),
        )?;

        token::transfer(
            ctx.accounts.into_transfer_to_pda_context(),
            amount,
        )?;

        if holder_mode {
            token::transfer(
                ctx.accounts.into_transfer_to_holder_mode_account(),
                HOLDER_MODE_AMOUNT,
            )?;
        } else if service_fee > 0 {
            token::transfer(
                ctx.accounts.into_transfer_to_service_account(),
                service_fee,
            )?;
        }

        if client_bond_amount > 0 {
            token::set_authority(
                ctx.accounts.into_set_authority_client_bond_context(),
                AuthorityType::AccountOwner,
                Some(*ctx.accounts.authority.to_account_info().key),
            )?;

            token::transfer(
                ctx.accounts.into_transfer_to_client_bond_account(),
                service_fee,
            )?;
        }

        if executor_bond_amount > 0 {
            token::set_authority(
                ctx.accounts.into_set_authority_executor_bond_context(),
                AuthorityType::AccountOwner,
                Some(*ctx.accounts.authority.to_account_info().key),
            )?;

            token::transfer(
                ctx.accounts.into_transfer_to_executor_bond_account(),
                executor_bond_amount,
            )?;
        }

        Ok(())
    }

    pub fn finish(
        ctx: Context<Finish>, 
        id: Vec<u8>
    ) -> Result<()> {

        if !ctx.accounts.deal_state.is_started {
            return Err(ErrorCode::NotStarted.into());
        }
        let seeds = &[
            &id, 
            &AUTHORITY_SEED[..],
            ctx.accounts.deal_state.client_key.as_ref(),
            ctx.accounts.deal_state.executor_key.as_ref(),
            &[ctx.accounts.deal_state.authority_bump]];

        token::transfer(
            ctx.accounts
                .into_transfer_to_executor_token_account_context()
                .with_signer(&[&seeds[..]]),
                ctx.accounts.deal_state.amount
        )?;
        
        if ctx.accounts.deal_state.checker_fee > 0 {
            token::transfer(
                ctx.accounts
                    .into_transfer_to_checker_token_account_context()
                    .with_signer(&[&seeds[..]]),
                    ctx.accounts.deal_state.checker_fee
            )?;
        } 

        token::close_account(
            ctx.accounts
                .into_close_context()
                .with_signer(&[&seeds[..]])
        )?;

        Ok(())
    }

    pub fn finish_with_bond(
        ctx: Context<FinishWithBond>, 
        id: Vec<u8>
    ) -> Result<()> {

        if !ctx.accounts.deal_state.is_started {
            return Err(ErrorCode::NotStarted.into());
        }
        let seeds = &[
            &id, 
            &AUTHORITY_SEED[..],
            ctx.accounts.deal_state.client_key.as_ref(),
            ctx.accounts.deal_state.executor_key.as_ref(),
            &[ctx.accounts.deal_state.authority_bump]];

        token::transfer(
            ctx.accounts
                .into_transfer_to_executor_token_account_context()
                .with_signer(&[&seeds[..]]),
                ctx.accounts.deal_state.amount
        )?;
        
        if ctx.accounts.deal_state.checker_fee > 0 {
            token::transfer(
                ctx.accounts
                    .into_transfer_to_checker_token_account_context()
                    .with_signer(&[&seeds[..]]),
                    ctx.accounts.deal_state.checker_fee
            )?;
        } 

        token::close_account(
            ctx.accounts
                .into_close_context()
                .with_signer(&[&seeds[..]])
        )?;

        Ok(())
    }

    pub fn cancel(
        ctx: Context<Cancel>,
        id: Vec<u8>
    ) -> Result<()> {

        if !ctx.accounts.deal_state.is_started {
            return Err(ErrorCode::NotStarted.into());
        }

        if ctx.accounts.deal_state.with_bond {
            return Err(ErrorCode::NeedCancelWithBond.into());
        }

        if ctx.accounts.deal_state.deadline_ts > 0 {
            let clock = Clock::get()?;
            let current_ts = clock.unix_timestamp;
            if current_ts > ctx.accounts.deal_state.deadline_ts {
                return Err(ErrorCode::DeadlineNotCome.into());
            }
        }

        let seeds = &[
            &id, 
            &AUTHORITY_SEED[..],
            ctx.accounts.deal_state.client_key.as_ref(),
            ctx.accounts.deal_state.executor_key.as_ref(),
            &[ctx.accounts.deal_state.authority_bump]];


        let amount = ctx.accounts.deal_state.amount + ctx.accounts.deal_state.checker_fee;
        
        token::transfer(
            ctx.accounts
                .into_transfer_to_client_token_account_context()
                .with_signer(&[&seeds[..]]),
                amount
        )?;

        token::close_account(
            ctx.accounts
                .into_close_context()
                .with_signer(&[&seeds[..]])
        )?;

        Ok(())
    }

    pub fn cancel_with_bond(
        ctx: Context<CancelWithBond>,
        id: Vec<u8>
    ) -> Result<()> {

        if !ctx.accounts.deal_state.is_started {
            return Err(ErrorCode::NotStarted.into());
        }

        if !ctx.accounts.deal_state.with_bond {
            return Err(ErrorCode::NeedCancelWithoutBond.into());
        }

        if ctx.accounts.deal_state.deadline_ts > 0 {
            let clock = Clock::get()?;
            let current_ts = clock.unix_timestamp;
            if current_ts > ctx.accounts.deal_state.deadline_ts {
                return Err(ErrorCode::DeadlineNotCome.into());
            }
        }

        let seeds = &[
            &id, 
            &AUTHORITY_SEED[..],
            ctx.accounts.deal_state.client_key.as_ref(),
            ctx.accounts.deal_state.executor_key.as_ref(),
            &[ctx.accounts.deal_state.authority_bump]];


        let amount = ctx.accounts.deal_state.amount + ctx.accounts.deal_state.checker_fee;
        
        token::transfer(
            ctx.accounts
                .into_transfer_to_client_token_account_context()
                .with_signer(&[&seeds[..]]),
                amount
        )?;

        if ctx.accounts.deal_state.client_bond_amount > 0 {
            token::transfer(
                ctx.accounts
                    .into_transfer_to_bond_client_token_account_context()
                    .with_signer(&[&seeds[..]]),
                    amount
            )?;
        }

        if ctx.accounts.deal_state.executor_bond_amount > 0 {
            token::transfer(
                ctx.accounts
                    .into_transfer_to_bond_client_token_account_context()
                    .with_signer(&[&seeds[..]]),
                    amount
            )?;
        }

        token::close_account(
            ctx.accounts
                .into_close_context()
                .with_signer(&[&seeds[..]])
        )?;

        Ok(())
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
        seeds = [&id, b"auth".as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.authority_bump,
    )]
    pub authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        seeds = [&id, b"deposit".as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.deposit_bump,
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
        constraint = (*initializer.to_account_info().key == deal_state.client_key || *initializer.to_account_info().key == deal_state.executor_key || *initializer.to_account_info().key == deal_state.checker_key || *initializer.to_account_info().key == deal_state.service_key),
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
        seeds = [&id, b"auth".as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.authority_bump,
    )]
    pub authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut,
        constraint = deal_state.deposit_key == *deposit_account.to_account_info().key,
        seeds = [&id, b"deposit".as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.deposit_bump,
        
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
        constraint = client_bond_account.owner.key() == deal_state.client_key
    )]
    pub client_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = executor_bond_account.owner.key() == deal_state.executor_key
    )]
    pub executor_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [&id, b"deposit_bond_client".as_ref(),  deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.client_bond_deposit_bump
    )]
    pub deposit_client_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [&id, b"deposit_bond_executor".as_ref(),  deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.executor_bond_deposit_bump,
    )]
    pub deposit_executor_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [&id, b"state".as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.bump,
        constraint = (*initializer.to_account_info().key == deal_state.client_key || *initializer.to_account_info().key == deal_state.executor_key || *initializer.to_account_info().key == deal_state.checker_key || *initializer.to_account_info().key == deal_state.service_key),
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
        seeds = [&id, b"auth".as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.authority_bump,
    )]
    pub authority: AccountInfo<'info>,
    #[account(
        mut,
        constraint = deal_state.deposit_key == *deposit_account.to_account_info().key,
        seeds = [&id, b"deposit".as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.deposit_bump
    )]
    pub deposit_account: Account<'info, TokenAccount>,
    #[account(
        seeds = [&id, b"holder_deposit".as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.holder_deposit_bump,
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
        seeds = [&id, b"auth".as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.authority_bump,
    )]
    pub authority: AccountInfo<'info>,
    #[account(
        mut,
        constraint = deal_state.deposit_key == *deposit_account.to_account_info().key,
        seeds = [&id, b"deposit".as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.deposit_bump
    )]
    pub deposit_account: Account<'info, TokenAccount>,
    #[account(
        seeds = [&id, b"holder_deposit".as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.holder_deposit_bump,
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
        constraint = client_bond_account.owner.key() == deal_state.client_key
    )]
    pub client_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = executor_bond_account.owner.key() == deal_state.executor_key
    )]
    pub executor_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [&id, b"deposit_bond_client".as_ref(),  deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.client_bond_deposit_bump
    )]
    pub deposit_client_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        seeds = [&id, b"deposit_bond_executor".as_ref(),  deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.executor_bond_deposit_bump,
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
    pub service_key: Pubkey,

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
    #[msg("The deal need cancel with bond")]
    NeedCancelWithBond,
    #[msg("The deal need cancel without bond")]
    NeedCancelWithoutBond,
}