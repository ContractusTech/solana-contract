use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, TokenAccount, Transfer};

use crate::{constants::*, errors::ErrorCode, state::DealState};

#[derive(Accounts)]
#[instruction(id: Vec<u8>)]
pub struct Finish<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
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
    pub token_program: AccountInfo<'info>,
}

// Finish
impl<'info> Finish<'info> {
    fn into_transfer_to_executor_token_account_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.deposit_account.to_account_info(),
            to: self.executor_token_account.to_account_info(),
            authority: self.authority.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_checker_token_account_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.deposit_account.to_account_info(),
            to: self.checker_token_account.to_account_info(),
            authority: self.authority.clone(),
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

pub fn handle(_ctx: Context<Finish>, _id: Vec<u8>) -> Result<()> {
    if !_ctx.accounts.deal_state.is_started {
        return Err(ErrorCode::NotStarted.into());
    }
    let seeds = &[
        &_id,
        &AUTHORITY_SEED[..],
        _ctx.accounts.deal_state.client_key.as_ref(),
        _ctx.accounts.deal_state.executor_key.as_ref(),
        &[_ctx.accounts.deal_state.authority_bump],
    ];

    token::transfer(
        _ctx.accounts
            .into_transfer_to_executor_token_account_context()
            .with_signer(&[&seeds[..]]),
        _ctx.accounts.deal_state.amount,
    )?;

    if _ctx.accounts.deal_state.checker_fee > 0 {
        token::transfer(
            _ctx.accounts
                .into_transfer_to_checker_token_account_context()
                .with_signer(&[&seeds[..]]),
            _ctx.accounts.deal_state.checker_fee,
        )?;
    }

    token::close_account(_ctx.accounts.into_close_context().with_signer(&[&seeds[..]]))?;

    Ok(())
}
