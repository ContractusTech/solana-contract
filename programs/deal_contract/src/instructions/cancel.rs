use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, TokenAccount, Transfer};

use crate::{constants::*, errors::ErrorCode, state::DealState};

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
        constraint = (*initializer.to_account_info().key == deal_state.client_key || *initializer.to_account_info().key == deal_state.executor_key || *initializer.to_account_info().key == deal_state.checker_key 
      || *initializer.to_account_info().key == SERVICE_ACCOUNT_ADDRESS),
        close = initializer
    )]
    pub deal_state: Box<Account<'info, DealState>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

// Cancel
impl<'info> Cancel<'info> {
    fn into_transfer_to_client_token_account_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.deposit_account.to_account_info(),
            to: self.client_token_account.to_account_info(),
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

pub fn handle(_ctx: Context<Cancel>, _id: Vec<u8>) -> Result<()> {
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
        &[_ctx.accounts.deal_state.authority_bump],
    ];

    let amount = _ctx.accounts.deal_state.amount + _ctx.accounts.deal_state.checker_fee;

    token::transfer(
        _ctx.accounts
            .into_transfer_to_client_token_account_context()
            .with_signer(&[&seeds[..]]),
        amount,
    )?;

    token::close_account(_ctx.accounts.into_close_context().with_signer(&[&seeds[..]]))?;

    Ok(())
}
