use anchor_lang::prelude::*;
use anchor_spl::{token::{self, CloseAccount, Token, TokenAccount, Transfer, Mint}, token_interface::spl_token_2022::cmp_pubkeys, associated_token::AssociatedToken};

use crate::{constants::*, state::{DealState, Checker, Bond}, 
    utils::{CheckerFeeTransfered, PaymentTransfered, BondsTransfered, AccountClosed, init_ata, check_ta}, errors::InvalidAccount};

#[derive(Accounts)]
pub struct Finish<'info> {
    /// CHECK:
    #[account(mut, signer, constraint = 
        cmp_pubkeys(&initializer.key, checker.key)
        || cmp_pubkeys(&initializer.key, client.key)
    )]
    pub initializer: AccountInfo<'info>,
    /// CHECK:
    #[account(address = deal_state.client_key)]
    pub client: AccountInfo<'info>,
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
    /// CHECK: may be uninitialized. check in access_control
    #[account(mut)]
    pub client_holder_ta: AccountInfo<'info>,
    #[account(init_if_needed, payer = initializer,
        associated_token::mint = deal_mint,
        associated_token::authority = executor,
    )]
    pub executor_deal_ta: Box<Account<'info, TokenAccount>>,
    /// CHECK: may be uninitialized. check in access_control
    #[account(mut)]
    pub checker_deal_ta: AccountInfo<'info>,

    /// CHECK: may be uninitialized. check in access_control
    #[account(mut)]
    pub deal_state_client_bond_ta: AccountInfo<'info>,
    /// CHECK: may be uninitialized. check in access_control
    #[account(mut)]
    pub deal_state_executor_bond_ta: AccountInfo<'info>,
    /// CHECK: may be uninitialized. check in access_control
    #[account(mut)]
    pub client_bond_ta: AccountInfo<'info>,
    /// CHECK: may be uninitialized. check in access_control
    #[account(mut)]
    pub executor_bond_ta: AccountInfo<'info>,

    #[account(address = deal_state.deal_token_mint)]
    pub deal_mint: Box<Account<'info, Mint>>,
    #[account(address = HOLDER_MINT)]
    pub holder_mint: Box<Account<'info, Mint>>,
    pub client_bond_mint: Box<Account<'info, Mint>>,
    pub executor_bond_mint: Box<Account<'info, Mint>>,

    /// CHECK: constant address
    #[account(mut, address = SERVICE_FEE_OWNER)]
    pub service_fee: AccountInfo<'info>,
    #[account(mut, constraint = 
        cmp_pubkeys(initializer.to_account_info().key, &deal_state.client_key) 
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

            match Account::<TokenAccount>::try_from(&ctx.accounts.client_holder_ta) {
                Ok(client_holder_ta) => {
                    if !cmp_pubkeys(&client_holder_ta.owner, ctx.accounts.client.to_account_info().key)
                    || !cmp_pubkeys(&client_holder_ta.mint, &HOLDER_MINT){
                        return Err(InvalidAccount::DealStateHolderTokenAccount.into())
                    }
                },
                Err(_) => {
                    init_ata(
                        &ctx.accounts.initializer, 
                        &ctx.accounts.holder_mint.to_account_info(), 
                        &ctx.accounts.client.to_account_info(), 
                        &ctx.accounts.client_holder_ta.to_account_info(), 
                        &ctx.accounts.token_program.to_account_info()
                    )?;
                }
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
                    init_ata(
                        &ctx.accounts.initializer, 
                        &ctx.accounts.deal_mint.to_account_info(), 
                        &ctx.accounts.checker.to_account_info(), 
                        &ctx.accounts.checker_deal_ta.to_account_info(), 
                        &ctx.accounts.token_program.to_account_info()
                    )?;
                }
            }
        }

        if let Some(Bond { mint, .. }) = ctx.accounts.deal_state.client_bond.as_ref() {
            if !cmp_pubkeys(&ctx.accounts.client_bond_mint.key(), mint) {
                return Err(InvalidAccount::ClientBondMint.into())
            }
            let deal_state_client_bond_ta = Account::<TokenAccount>::try_from(&ctx.accounts.deal_state_client_bond_ta)?;
            check_ta(&deal_state_client_bond_ta, &ctx.accounts.client_bond_mint.key(), &ctx.accounts.deal_state.key())?;
            match Account::<TokenAccount>::try_from(&ctx.accounts.client_bond_ta) {
                Ok(client_bond_ta) => {
                    check_ta(&client_bond_ta, &ctx.accounts.client_bond_mint.key(), ctx.accounts.client.key)?;
                },
                Err(_) => {
                    init_ata(
                        &ctx.accounts.initializer, 
                        &ctx.accounts.client_bond_mint.to_account_info(), 
                        &ctx.accounts.client,
                        &ctx.accounts.client_bond_ta, 
                        &ctx.accounts.token_program.to_account_info() 
                    )?;
                }
            }
        };

        if let Some(Bond { mint, .. }) = ctx.accounts.deal_state.executor_bond.as_ref() {
            if !cmp_pubkeys(&ctx.accounts.executor_bond_mint.key(), mint) {
                return Err(InvalidAccount::ExecutorBondMint.into())
            }
            let deal_state_executor_bond_ta = Account::<TokenAccount>::try_from(&ctx.accounts.deal_state_executor_bond_ta)?;
            check_ta(&deal_state_executor_bond_ta, &ctx.accounts.executor_bond_mint.key(), &ctx.accounts.deal_state.key())?;
            match Account::<TokenAccount>::try_from(&ctx.accounts.executor_bond_ta) {
                Ok(executor_bond_ta) => {
                    check_ta(&executor_bond_ta, &ctx.accounts.executor_bond_mint.key(), ctx.accounts.executor.key)?;
                },
                Err(_) => {
                    init_ata(
                        &ctx.accounts.initializer, 
                        &ctx.accounts.executor_bond_mint.to_account_info(), 
                        &ctx.accounts.executor,
                        &ctx.accounts.executor_bond_ta, 
                        &ctx.accounts.token_program.to_account_info() 
                    )?;
                }
            }
        };

        Ok(())
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
                destination: self.service_fee.clone(),
                authority: self.deal_state.to_account_info(),
        }, &[&self.deal_state.seeds()[..]]))?;
        Ok(AccountClosed)
    }
}

#[allow(dead_code)]
struct Checklist {
    checker_fee_transfered: CheckerFeeTransfered,
    payment_transfered: PaymentTransfered,
    bonds_transfered: BondsTransfered,
    deal_state_deal_ta_closed: AccountClosed,
}


#[access_control(Finish::check_accounts(&ctx))]
pub fn handle(ctx: Context<Finish>) -> Result<()> {
    let payment_transfered = ctx.accounts.transfer_payment()?;
    let checker_fee_transfered = ctx.accounts.transfer_checker_fee()?;
    let bonds_transfered = ctx.accounts.transfer_bonds()?;

    let deal_state_deal_ta_closed = ctx.accounts.close_deal_state_deal_ta()?;

    Checklist {
        checker_fee_transfered,
        payment_transfered,
        bonds_transfered,
        deal_state_deal_ta_closed,
    };
    
    Ok(())
}
