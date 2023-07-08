use anchor_lang::prelude::*;
use anchor_spl::{token::Token, token_interface::spl_token_2022::cmp_pubkeys};

use crate::{
    constants::*,
    errors::InvalidAccount,
    state::{Checker, DealState},
};

#[derive(Accounts)]
pub struct UpdateChecker<'info> {
    /// CHECK:
    #[account(mut, signer)]
    pub initializer: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, address = deal_state.client_key)]
    pub client: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, address = deal_state.executor_key)]
    pub executor: AccountInfo<'info>,
    /// CHECK:
    #[account(signer)]
    pub current_checker: AccountInfo<'info>,
    /// CHECK:
    #[account(signer)]
    pub new_checker: AccountInfo<'info>,

    #[account(mut)]
    pub deal_state: Box<Account<'info, DealState>>,
    pub token_program: Program<'info, Token>,
}

pub fn handle(ctx: Context<UpdateChecker>, new_checker_fee: u64) -> Result<()> {
    if !cmp_pubkeys(ctx.accounts.initializer.key, &SERVICE_ACCOUNT_ADDRESS) {
        require!(ctx.accounts.client.is_signer, ErrorCode::AccountNotSigner);
        require!(ctx.accounts.executor.is_signer, ErrorCode::AccountNotSigner);

        if let Some(Checker { checker_key, .. }) = ctx.accounts.deal_state.checker {
            require_keys_eq!(
                checker_key,
                ctx.accounts.current_checker.key(),
                InvalidAccount::Checker
            );
        }
    };

    ctx.accounts.deal_state.checker = Some(Checker {
        checker_fee: new_checker_fee,
        checker_key: ctx.accounts.new_checker.key(),
    });

    Ok(())
}
