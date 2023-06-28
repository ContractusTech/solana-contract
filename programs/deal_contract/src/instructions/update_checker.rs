use anchor_lang::prelude::*;

use crate::{constants::*, errors::ErrorCode, state::DealState};

#[derive(Accounts)]
#[instruction(id: Vec<u8>)]
pub struct UpdateChecker<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut, 
        signer,
        constraint = SERVICE_ACCOUNT_ADDRESS == *initializer.to_account_info().key,
    )]
    pub initializer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut)]
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
    pub token_program: AccountInfo<'info>,
}

pub fn handle(_ctx: Context<UpdateChecker>, _id: Vec<u8>) -> Result<()> {
    // TODO: -
    return Err(ErrorCode::NotImplemented.into());
}
