use std::cell::Ref;
use std::cmp;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Mint, SetAuthority, TokenAccount, Transfer};
use spl_token::instruction::AuthorityType;
use std::convert::Into;
use std::convert::TryInto;
use std::str::FromStr;

declare_id!("9kpWdyR2qtNT21MhLRTBbT21v5thz9hhB3zaPUhr6tbE");

static SERVICE_TOKEN_ADDRESS: &'static str = "CyhjLfsfDz7rtszqBGaHiFrBbck2LNKEXQkywqNrGVyw"; // NEED CHECK
static HOLDER_MODE_AMOUNT: u64 = 10000000000000; // NEED CHECK

#[program]
pub mod deal_contract {
    use super::*;

    const AUTHORITY_SEED: &[u8] = b"auth";
    
    pub fn initialize(
        ctx: Context<Initialize>,
        _vault_account_bump: u8,
        _state_account_bump: u8,
        id: Vec<u8>,
        amount: u64,
        service_fee: u64,
        checker_fee: u64
    ) -> Result<()> {

        if ctx.accounts.deal_state.is_started {
            return Err(ErrorCode::AlreadyStarted.into());
        }

        if amount == 0 {
            return Err(ErrorCode::AmountTooLow.into());
        }

        let FREE_COMISSION_TOKEN: Pubkey = Pubkey::from_str(SERVICE_TOKEN_ADDRESS).unwrap();

        if service_fee == 0 {
            if *ctx.accounts.mint.to_account_info().key != FREE_COMISSION_TOKEN {
                 return Err(ErrorCode::FeeIsTooLow.into());
            }
            if ctx.accounts.client_service_token_account.mint == FREE_COMISSION_TOKEN 
            && ctx.accounts.client_token_account.mint == FREE_COMISSION_TOKEN 
            && ctx.accounts.client_service_token_account.amount < HOLDER_MODE_AMOUNT {
                return Err(ErrorCode::HolderModeUnavailable.into());
            }
        }

        ctx.accounts.deal_state.is_started = true;

        ctx.accounts.deal_state.client_key = *ctx.accounts.client.key;
        ctx.accounts.deal_state.executor_key = *ctx.accounts.executor.to_account_info().key;
        ctx.accounts.deal_state.checker_key = *ctx.accounts.checker.to_account_info().key;
        ctx.accounts.deal_state.deposit_key = *ctx.accounts.deposit_account.to_account_info().key;
        
        ctx.accounts.deal_state.client_token_account_key = *ctx.accounts.client_token_account.to_account_info().key;
        ctx.accounts.deal_state.executor_token_account_key = *ctx.accounts.executor_token_account.to_account_info().key;
        ctx.accounts.deal_state.checker_token_account_key = *ctx.accounts.checker_token_account.to_account_info().key;
        ctx.accounts.deal_state.service_key = *ctx.accounts.checker_token_account.to_account_info().key;

        ctx.accounts.deal_state.checker_fee = checker_fee;
        ctx.accounts.deal_state.amount = amount;
        
        let (authority, authority_bump) =
            Pubkey::find_program_address(&[&id, &AUTHORITY_SEED], ctx.program_id);

        token::set_authority(
            ctx.accounts.into_set_authority_context(),
            AuthorityType::AccountOwner,
            Some(authority),
        )?;

        ctx.accounts.deal_state.authority_deposit = authority;

        token::transfer(
            ctx.accounts.into_transfer_to_service_account(),
            service_fee,
        )?;

        token::transfer(
            ctx.accounts.into_transfer_to_pda_context(),
            amount + checker_fee,
        )?;

        Ok(())
    }

    pub fn finish(
        ctx: Context<Finish>, 
        id: Vec<u8>
    ) -> Result<()> {

        if !ctx.accounts.deal_state.is_started {
            return Err(ErrorCode::NotStarted.into());
        }

        let (_authority, _authority_bump) =
            Pubkey::find_program_address(&[&id, &AUTHORITY_SEED], ctx.program_id);

        let authority_seeds = &[&id, &AUTHORITY_SEED[..], &[_authority_bump]];
        let amount = ctx.accounts.deal_state.amount + ctx.accounts.deal_state.checker_fee;

        token::transfer(
            ctx.accounts
                .into_transfer_to_executor_token_account_context()
                .with_signer(&[&authority_seeds[..]]),
                ctx.accounts.deal_state.amount,
        )?;
        
        if ctx.accounts.deal_state.checker_fee > 0 {
            token::transfer(
                ctx.accounts
                    .into_transfer_to_checker_token_account_context()
                    .with_signer(&[&authority_seeds[..]]),
                    ctx.accounts.deal_state.checker_fee,
            )?;
        } 

        token::close_account(
            ctx.accounts
                .into_close_context()
                .with_signer(&[&authority_seeds[..]]),
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

        let (_authority, _authority_bump) =
            Pubkey::find_program_address(&[&id, &AUTHORITY_SEED], ctx.program_id);

        let authority_seeds = &[&id, &AUTHORITY_SEED[..], &[_authority_bump]];
        let amount = ctx.accounts.deal_state.amount + ctx.accounts.deal_state.checker_fee;
        
        token::transfer(
            ctx.accounts
                .into_transfer_to_client_token_account_context()
                .with_signer(&[&authority_seeds[..]]),
                amount,
        )?;

        token::close_account(
            ctx.accounts
                .into_close_context()
                .with_signer(&[&authority_seeds[..]]),
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(vault_account_bump: u8, state_account_bump: u8, id: Vec<u8>, amount: u64, service_fee: u64, checker_fee: u64)]
pub struct Initialize<'info> {
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
        constraint = client_token_account.amount >= (amount + service_fee + checker_fee)
    )]
    pub client_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        constraint = client_service_token_account.owner.key() == *client.to_account_info().key
    )]
    pub client_service_token_account: Box<Account<'info, TokenAccount>>,
     /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub executor_token_account: Box<Account<'info, TokenAccount>>,
     /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub checker_token_account: Box<Account<'info, TokenAccount>>,
    pub mint: Account<'info, Mint>,
    #[account(
        init,
        seeds = [&id, b"deposit".as_ref()],
        bump,
        payer = payer,
        token::mint = mint,
        token::authority = client,
    )]
    pub deposit_account: Account<'info, TokenAccount>,
   
    #[account(
        init,
        seeds = [&id, b"state".as_ref()],
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
pub struct Cancel<'info> {    
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub initializer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub deposit_account: Account<'info, TokenAccount>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub client_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = (*initializer.to_account_info().key == deal_state.client_key || *initializer.to_account_info().key == deal_state.executor_key || *initializer.to_account_info().key == deal_state.checker_key || *initializer.to_account_info().key == deal_state.service_key),
        constraint = deal_state.deposit_key == *deposit_account.to_account_info().key,
        constraint = *authority.to_account_info().key == deal_state.authority_deposit,
        constraint = *client_token_account.to_account_info().key == deal_state.client_token_account_key,
        close = initializer
    )]
    pub deal_state: Box<Account<'info, DealState>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>
}

#[derive(Accounts)]
pub struct Finish<'info> { 
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub initializer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub authority: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub deposit_account: Account<'info, TokenAccount>,
     /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub executor_token_account: Account<'info, TokenAccount>,
     /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
    pub checker_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = (*initializer.to_account_info().key == deal_state.client_key || *initializer.to_account_info().key == deal_state.checker_key),
        constraint = deal_state.deposit_key == *deposit_account.to_account_info().key,
        constraint = *authority.to_account_info().key == deal_state.authority_deposit,
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
    pub client_key: Pubkey,
    pub client_token_account_key: Pubkey,
    pub executor_key: Pubkey,
    pub executor_token_account_key: Pubkey,
    pub checker_key: Pubkey,
    pub checker_token_account_key: Pubkey,
    pub deposit_key: Pubkey,
    pub authority_deposit: Pubkey,
    pub service_key: Pubkey,
    pub amount: u64,
    pub checker_fee: u64,
    pub is_started: bool
}

impl DealState {
    pub fn space() -> usize {
        8 + 32 + 32 + 32 + 32 + 32 + 32 + 32 + 32 + 32 + 8 + 8 + 1
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

    fn into_set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.deposit_account.to_account_info(),
            current_authority: self.client.clone(),
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
            authority: self.authority.clone(),
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
            authority: self.authority.clone(),
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
    AmountTooLow
}