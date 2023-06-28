use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::{constants::*, errors::ErrorCodes, state::DealState};

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
        seeds = [&id, DEAL_STATE_SEED.as_ref(), deal_state.client_key.as_ref(), deal_state.executor_key.as_ref()],
        bump = deal_state.bump
    )]
    pub deal_state: Box<Account<'info, DealState>>,
    pub token_program: Program<'info, Token>,
}

pub fn handle(_ctx: Context<UpdateChecker>, _id: Vec<u8>) -> Result<()> {
    // TODO: -
    return Err(ErrorCodes::NotImplemented.into());
}
