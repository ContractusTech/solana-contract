use anchor_lang::prelude::*;
use anchor_spl::{token::{self, CloseAccount, TokenAccount, Transfer, Token, Mint}, token_interface::spl_token_2022::cmp_pubkeys, associated_token::AssociatedToken};

use crate::{constants::*, errors::{ErrorCodes, InvalidAccount}, 
    state::{DealState, Checker, Bond }, 
    utils::{BondsTransfered, DeadlineChecked, AccountClosed, DepositTransfered }};

#[derive(Accounts)]
pub struct Cancel<'info> {
    /// CHECK: 
    #[account(mut, signer, constraint = cmp_pubkeys(&initializer.key, &deal_state.client_key)
        || cmp_pubkeys(&initializer.key, &deal_state.executor_key)
        || cmp_pubkeys(&initializer.key, checker.key)
    )]
    pub initializer: AccountInfo<'info>,
    /// CHECK: check is performed in access_control
    pub checker: AccountInfo<'info>,

    #[account(
        mut,
        constraint = cmp_pubkeys(&deal_state.deal_token_mint, &deal_state_deal_ta.mint),
        constraint = cmp_pubkeys(&deal_state_deal_ta.owner, &deal_state.key())
    )]
    pub deal_state_deal_ta: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = cmp_pubkeys(&client_deal_ta.mint, &deal_state.deal_token_mint),
        constraint = cmp_pubkeys(&client_deal_ta.owner, &deal_state.client_key)
    )]
    pub client_deal_ta: Box<Account<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = initializer,
        associated_token::mint = deal_mint,
        associated_token::authority = if let Some(Checker{checker_key, ..}) = deal_state.checker {
            checker.as_ref()
        } else { 
            initializer.as_ref() 
        },
    )]
    pub checker_deal_ta: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub client_bond_ta: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub executor_bond_ta: Box<Account<'info, TokenAccount>>,

    pub deal_state_client_bond_ta: Box<Account<'info, TokenAccount>>,
    pub deal_state_executor_bond_ta: Box<Account<'info, TokenAccount>>,

    #[account(constraint = cmp_pubkeys(&deal_mint.key(), &deal_state.deal_token_mint.key()))]
    pub deal_mint: Box<Account<'info, Mint>>,
    pub client_bond_mint: Box<Account<'info, Mint>>,
    pub executor_bond_mint: Box<Account<'info, Mint>>,
    
    #[account(mut, constraint = 
        (*initializer.key == deal_state.client_key 
            || *initializer.key == deal_state.executor_key 
            || if let Some(Checker{checker_key, ..}) = deal_state.checker.as_ref() {
                cmp_pubkeys(initializer.key, &checker_key) } else { false }
            || *initializer.key == SERVICE_ACCOUNT_ADDRESS),
        close = initializer
    )]
    pub deal_state: Box<Account<'info, DealState>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

#[allow(dead_code)]
struct Checklist {
    deadline_checked: DeadlineChecked,
    deposit_transfered: DepositTransfered,
    bonds_transfered: BondsTransfered,
    deal_state_deal_ta_closed: AccountClosed,
}


impl<'info> Cancel<'info> {
    fn check_accounts(ctx: &Context<Cancel>) -> Result<()> {
        match ctx.accounts.deal_state.checker.as_ref() {
            Some(Checker{checker_key, ..}) => {
                if !cmp_pubkeys(ctx.accounts.checker.as_ref().key, &checker_key) {
                    return Err(InvalidAccount::Checker.into())
                };
                if !cmp_pubkeys(&ctx.accounts.checker_deal_ta.owner, &checker_key)
                || !cmp_pubkeys(&ctx.accounts.checker_deal_ta.mint, &ctx.accounts.deal_mint.key()) {
                    return Err(InvalidAccount::CheckerDealTokenAccount.into())
                };
            }
            _ => {}
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
        if self.deal_state.deadline_ts.is_some() && !self.deal_state.deadline_expired() {
            return Err(ErrorCodes::DeadlineNotExpired.into());
        }
        Ok(DeadlineChecked)
    }
    
    fn transfer_deposit(&self) -> Result<DepositTransfered> {
        let checker_fee = if let Some(Checker{checker_fee,..}) = self.deal_state.checker {checker_fee} else {0};
        token::transfer(
            CpiContext::new_with_signer(self.token_program.to_account_info(), 
                Transfer { from: self.deal_state_deal_ta.to_account_info(), 
                    to: self.client_deal_ta.to_account_info(), 
                    authority:  self.deal_state.to_account_info()}, 
                &[&self.deal_state.seeds()[..]]
        ), self.deal_state.amount + checker_fee)?;
        Ok(DepositTransfered)
        
    }

    fn transfer_bonds(&mut self) -> Result<BondsTransfered> {
        if let Some(Bond { amount, .. }) = self.deal_state.client_bond.as_ref() {
            if *amount > 0 {
                anchor_spl::token::transfer(CpiContext::new(
                    self.token_program.to_account_info(),
                    token::Transfer {
                        from: self.deal_state_client_bond_ta.to_account_info(),
                        to: self.client_bond_ta.to_account_info(),
                        authority: self.deal_state.to_account_info(),
                    },
                ).with_signer(&[&self.deal_state.seeds()[..]]), *amount)?;
                
                self.close_deal_state_ta(&*self.deal_state_client_bond_ta)?;
            }
        }
        if let Some(Bond { amount, .. }) = self.deal_state.executor_bond.as_ref() {
            if *amount > 0 {
                anchor_spl::token::transfer(CpiContext::new(
                    self.token_program.to_account_info(),
                    token::Transfer {
                        from: self.deal_state_client_bond_ta.to_account_info(),
                        to: self.executor_bond_ta.to_account_info(),
                        authority: self.deal_state.to_account_info(),
                    },
                ).with_signer(&[&self.deal_state.seeds()[..]]), *amount)?;
                
                self.close_deal_state_ta(&*self.deal_state_executor_bond_ta)?;
            }
        }
        Ok(BondsTransfered)
    }

    fn close_deal_state_ta(&self, deal_state_ta: &impl ToAccountInfo<'info>) -> Result<AccountClosed> {
        token::close_account(CpiContext::new(self.token_program.to_account_info(), CloseAccount {
            account: deal_state_ta.to_account_info(),
            destination: self.initializer.to_account_info(),
            authority: self.deal_state.to_account_info().clone(),
        }).with_signer(&[&self.deal_state.seeds()[..]]))?;
        Ok(AccountClosed)   
    }
}

#[access_control(Cancel::check_accounts(&ctx))]
pub fn handle(ctx: Context<Cancel>) -> Result<()> {
    let deadline_checked = ctx.accounts.check_deadline()?;
    let deposit_transfered = ctx.accounts.transfer_deposit()?;
    let bonds_transfered = ctx.accounts.transfer_bonds()?;

    let deal_state_deal_ta_closed = ctx.accounts.close_deal_state_ta(&*ctx.accounts.deal_state_deal_ta)?;

    Checklist {
        deadline_checked,
        deposit_transfered,
        bonds_transfered,
        deal_state_deal_ta_closed,
    };

    Ok(())
}
