use anchor_lang::prelude::*;
use anchor_spl::{token::{self, CloseAccount, Token, TokenAccount, Transfer, Mint}, token_interface::spl_token_2022::cmp_pubkeys, associated_token::AssociatedToken};

use crate::{constants::*, state::{DealState, Checker, Bond}, 
    utils::{CheckerFeeTransfered, PaymentTransfered, BondsTransfered, AccountClosed, DeadlineChecked}, errors::{InvalidAccount, ErrorCodes}};

#[derive(Accounts)]
pub struct Finish<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer, constraint = 
        cmp_pubkeys(&initializer.key, executor.key) || cmp_pubkeys(&initializer.key, checker.key)
    )]
    pub initializer: AccountInfo<'info>,
    /// CHECK:
    #[account(address = deal_state.executor_key)]
    pub executor: AccountInfo<'info>,
    /// CHECK: check in access_control
    pub checker: AccountInfo<'info>,
    
    #[account(mut,
        constraint = cmp_pubkeys(&deal_state.deal_token_mint, &deal_state_deal_ta.mint),
        constraint = cmp_pubkeys(&deal_state_deal_ta.owner, &deal_state.key())
    )]
    pub deal_state_deal_ta: Box<Account<'info, TokenAccount>>,
    /// CHECK: may be uninitialized. check in access_control
    #[account(mut)]
    pub deal_state_holder_ta: AccountInfo<'info>,
    #[account(mut,
        constraint = cmp_pubkeys(&executor_deal_ta.owner, &deal_state.executor_key),
        constraint = cmp_pubkeys(&executor_deal_ta.mint, &deal_state.deal_token_mint)
    )]
    pub executor_deal_ta: Box<Account<'info, TokenAccount>>,
    /// CHECK: may be uninitialized. check in access_control
    #[account(mut)]
    pub checker_deal_ta: AccountInfo<'info>,

    #[account(mut)]
    pub deal_state_client_bond_ta: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub deal_state_executor_bond_ta: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub client_bond_ta: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub executor_bond_ta: Box<Account<'info, TokenAccount>>,

    pub deal_mint: Box<Account<'info, Mint>>,
    pub client_bond_mint: Box<Account<'info, Mint>>,
    pub executor_bond_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = cmp_pubkeys(initializer.to_account_info().key, &deal_state.client_key) 
            || if let Some(Checker{checker_key, ..}) = deal_state.checker.as_ref() { 
                cmp_pubkeys(initializer.to_account_info().key, &checker_key)} else { true },
        close = initializer
    )]
    pub deal_state: Box<Account<'info, DealState>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> Finish<'info> {
    fn check_accounts(ctx: &Context<Finish>) -> Result<()> {
        if ctx.accounts.deal_state.holder_mode.is_some() {
            let deal_state_holder_ta = Account::<TokenAccount>::try_from(&ctx.accounts.deal_state_holder_ta)
                .map_err(|_|InvalidAccount::DealStateHolderTokenAccount)?;
            if !cmp_pubkeys(&deal_state_holder_ta.owner, ctx.accounts.deal_state.to_account_info().key)
            || !cmp_pubkeys(&deal_state_holder_ta.mint, &HOLDER_MINT){
                return Err(InvalidAccount::DealStateHolderTokenAccount.into())
            }
        }

        if let Some(Checker{checker_key, ..}) = ctx.accounts.deal_state.checker.as_ref() {
            match Account::<TokenAccount>::try_from(&ctx.accounts.checker_deal_ta) {
                Ok(checker_deal_ta) => {
                    if !cmp_pubkeys(ctx.accounts.checker.as_ref().key, &checker_key) {
                        return Err(InvalidAccount::Checker.into())
                    };
                    if !cmp_pubkeys(&checker_deal_ta.owner, &checker_key)
                    || !cmp_pubkeys(&checker_deal_ta.mint, &ctx.accounts.deal_mint.key()) {
                        return Err(InvalidAccount::CheckerDealTokenAccount.into())
                    };
                },
                Err(_) => {

                    solana_program::program::invoke(
                        &spl_associated_token_account::instruction::create_associated_token_account(
                        ctx.accounts.initializer.key,
                        ctx.accounts.checker.key,
                        ctx.accounts.deal_mint.to_account_info().key,
                        ctx.accounts.token_program.key,
                    ),
                        &[
                            ctx.accounts.initializer.to_account_info(),
                            ctx.accounts.checker.to_account_info(),
                            ctx.accounts.deal_mint.to_account_info(),
                            ctx.accounts.checker_deal_ta.to_account_info(),
                            ctx.accounts.token_program.to_account_info(),
                        ],
                    )?;
                }
            }
        }

        if let Some(Bond { mint, .. }) = ctx.accounts.deal_state.client_bond.as_ref() {
            if !cmp_pubkeys(&ctx.accounts.client_bond_ta.mint, &mint) {
                return Err(InvalidAccount::ClientBondMint.into())
            };
            if !cmp_pubkeys(&ctx.accounts.client_bond_ta.owner, &ctx.accounts.deal_state.client_key) {
                return Err(InvalidAccount::ClientBondTokenAccount.into())
            };
            if !cmp_pubkeys(&ctx.accounts.deal_state_client_bond_ta.mint, &mint) {
                return Err(InvalidAccount::DealStateClientBondMint.into())
            };
            if !cmp_pubkeys(&ctx.accounts.deal_state_client_bond_ta.owner, &ctx.accounts.deal_state.client_key) {
                return Err(InvalidAccount::DealStateClientBondTokenAccount.into())
            };
        };

        if let Some(Bond { mint, .. }) = ctx.accounts.deal_state.executor_bond.as_ref() {
            if !cmp_pubkeys(&ctx.accounts.executor_bond_ta.mint, &mint) {
                return Err(InvalidAccount::ExecutorBondMint.into())
            };
            if !cmp_pubkeys(&ctx.accounts.executor_bond_ta.owner, &ctx.accounts.deal_state.executor_key) {
                return Err(InvalidAccount::ExecutorBondTokenAccount.into())
            };
            if !cmp_pubkeys(&ctx.accounts.deal_state_executor_bond_ta.mint, &mint) {
                return Err(InvalidAccount::DealStateExecutorBondMint.into())
            };
            if !cmp_pubkeys(&ctx.accounts.deal_state_executor_bond_ta.owner, &ctx.accounts.deal_state.executor_key) {
                return Err(InvalidAccount::DealStateExecutorBondTokenAccount.into())
            };
        };

        Ok(())
    }

    fn check_deadline(&self) -> Result<DeadlineChecked> {
        if self.deal_state.deadline_ts.is_some() && !self.deal_state.deadline_expired() && cmp_pubkeys(&self.initializer.key, &self.checker.key) {
            return Err(ErrorCodes::DeadlineNotExpired.into())
        } else {
            Ok(DeadlineChecked)
        }
    }

    fn transfer_payment(&self) -> Result<PaymentTransfered> {
        token::transfer(CpiContext::new_with_signer(self.token_program.to_account_info(), Transfer {
            from: self.deal_state_deal_ta.to_account_info(),
            to: self.executor_deal_ta.to_account_info(),
            authority: self.deal_state.to_account_info(),
        }, &[&self.deal_state.seeds()[..]]), self.deal_state.amount)?;
        Ok(PaymentTransfered)
    }

    fn transfer_checker_fee(&self) -> Result<CheckerFeeTransfered> {
        if let Some(Checker { checker_fee, .. }) = self.deal_state.checker {
            if checker_fee > 0 {
                token::transfer(CpiContext::new_with_signer(self.token_program.to_account_info(), Transfer {
                    from: self.deal_state_deal_ta.to_account_info(),
                    to: self.checker_deal_ta.to_account_info(),
                    authority: self.deal_state.to_account_info(),
                }, &[&self.deal_state.seeds()[..]]), checker_fee)?;
            }
        }
        Ok(CheckerFeeTransfered)
    }

    fn transfer_bonds(&self) -> Result<BondsTransfered> {
        if let Some(Bond{ amount, .. }) = self.deal_state.client_bond {
            if amount > 0 {
                token::transfer(CpiContext::new_with_signer(self.token_program.to_account_info(), Transfer {
                    from: self.deal_state_client_bond_ta.to_account_info(),
                    to: self.client_bond_ta.to_account_info(),
                    authority: self.deal_state.to_account_info(),
                }, &[&self.deal_state.seeds()[..]]), amount)?;
            }
        }
        if let Some(Bond{ amount, .. }) = self.deal_state.executor_bond {
            if amount > 0 {
                token::transfer(CpiContext::new_with_signer(self.token_program.to_account_info(), Transfer {
                    from: self.deal_state_executor_bond_ta.to_account_info(),
                    to: self.executor_bond_ta.to_account_info(),
                    authority: self.deal_state.to_account_info(),
                }, &[&self.deal_state.seeds()[..]]), amount)?;
            }
        }
        Ok(BondsTransfered)
    }

    fn close_deal_state_deal_ta(&self) -> Result<AccountClosed> {
        token::close_account(
            CpiContext::new_with_signer(self.token_program.to_account_info(), CloseAccount {
                account: self.deal_state_deal_ta.to_account_info(),
                destination: self.initializer.to_account_info(),
                authority: self.deal_state.to_account_info(),
        }, &[&self.deal_state.seeds()[..]]))?;
        Ok(AccountClosed)
    }
}

#[allow(dead_code)]
struct Checklist {
    deadline_checked: DeadlineChecked,
    checker_fee_transfered: CheckerFeeTransfered,
    payment_transfered: PaymentTransfered,
    bonds_transfered: BondsTransfered,
    deal_state_deal_ta_closed: AccountClosed,
}


#[access_control(Finish::check_accounts(&ctx))]
pub fn handle(ctx: Context<Finish>) -> Result<()> {
    let deadline_checked = ctx.accounts.check_deadline()?;
    let payment_transfered = ctx.accounts.transfer_payment()?;
    let checker_fee_transfered = ctx.accounts.transfer_checker_fee()?;
    let bonds_transfered = ctx.accounts.transfer_bonds()?;

    let deal_state_deal_ta_closed = ctx.accounts.close_deal_state_deal_ta()?;

    Checklist {
        deadline_checked,
        checker_fee_transfered,
        payment_transfered,
        bonds_transfered,
        deal_state_deal_ta_closed,
    };
    
    Ok(())
}
