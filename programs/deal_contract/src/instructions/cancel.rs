use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, CloseAccount, Mint, Token, TokenAccount, Transfer},
    token_interface::spl_token_2022::cmp_pubkeys,
};

use crate::{
    constants::*,
    errors::{ErrorCodes, InvalidAccount},
    state::{Bond, Checker, DealState},
    utils::{
        check_ta, init_ata, AccountClosed, BondsTransfered, CheckerFeeTransfered, DeadlineChecked,
        DepositTransfered,
    },
};

#[derive(Accounts)]
pub struct Cancel<'info> {
    /// CHECK: check is performed in access_control
    #[account(mut, signer)]
    pub initializer: AccountInfo<'info>,
    /// CHECK: check is performed in access_control
    pub checker: AccountInfo<'info>,
    /// CHECK:
    #[account(address = deal_state.client_key)]
    pub client: AccountInfo<'info>,
    /// CHECK:
    #[account(address = deal_state.executor_key)]
    pub executor: AccountInfo<'info>,
    /// CHECK:
    #[account(mut, signer)]
    pub payer: AccountInfo<'info>,

    #[account(mut,
        constraint = cmp_pubkeys(&deal_state.deal_token_mint, &deal_state_deal_ta.mint),
        constraint = cmp_pubkeys(&deal_state_deal_ta.owner, &deal_state.key())
    )]
    pub deal_state_deal_ta: Box<Account<'info, TokenAccount>>,
    #[account(init_if_needed, payer = payer,
        associated_token::mint = deal_mint,
        associated_token::authority = client,
    )]
    pub client_deal_ta: Box<Account<'info, TokenAccount>>,

    /// CHECK: in access_control. may be uninitialized.
    #[account(mut)]
    pub checker_deal_ta: AccountInfo<'info>,

    /// CHECK: in access_control. may be uninitialized.
    #[account(mut)]
    pub client_bond_ta: AccountInfo<'info>,
    /// CHECK: in access_control. may be uninitialized.
    #[account(mut)]
    pub executor_bond_ta: AccountInfo<'info>,

    /// CHECK: in access_control
    #[account(mut)]
    pub deal_state_client_bond_ta: AccountInfo<'info>,
    /// CHECK: in access_control
    #[account(mut)]
    pub deal_state_executor_bond_ta: AccountInfo<'info>,

    #[account(constraint = cmp_pubkeys(&deal_mint.key(), &deal_state.deal_token_mint))]
    pub deal_mint: Box<Account<'info, Mint>>,
    /// CHECK: in transfer_bonds
    pub client_bond_mint: AccountInfo<'info>,
    /// CHECK: in transfer_bonds
    pub executor_bond_mint: AccountInfo<'info>,

    /// CHECK: constant address
    #[account(mut, address = SERVICE_FEE_OWNER)]
    pub service_fee: AccountInfo<'info>,
    #[account(mut, close = initializer)]
    pub deal_state: Box<Account<'info, DealState>>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

enum Initializer {
    Client,
    Executor,
    Checker,
    Service,
}

impl<'info> TryFrom<&Cancel<'info>> for Initializer {
    type Error = anchor_lang::error::Error;
    fn try_from(value: &Cancel<'info>) -> Result<Self> {
        let initializer_key = &value.initializer.key;
        if cmp_pubkeys(initializer_key, value.checker.key) {
            Ok(Initializer::Checker)
        } else if cmp_pubkeys(&initializer_key, value.client.key) {
            Ok(Initializer::Client)
        } else if cmp_pubkeys(&initializer_key, value.executor.key) {
            Ok(Initializer::Executor)
        } else if cmp_pubkeys(&initializer_key, &SERVICE_ACCOUNT_ADDRESS) {
            Ok(Initializer::Service)
        } else {
            Err(InvalidAccount::Initializer.into())
        }
    }
}

#[allow(dead_code)]
struct Checklist {
    deadline_checked: DeadlineChecked,
    checker_fee_transfered: CheckerFeeTransfered,
    deposit_transfered: DepositTransfered,
    bonds_transfered: BondsTransfered,
    deal_state_deal_ta_closed: AccountClosed,
}

impl<'info> Cancel<'info> {
    fn check_accounts(ctx: &Context<Cancel>) -> Result<()> {
        if let Some(Checker { checker_key, .. }) = ctx.accounts.deal_state.checker.as_ref() {
            if !cmp_pubkeys(ctx.accounts.checker.as_ref().key, &checker_key) {
                return Err(InvalidAccount::Checker.into());
            };

            match Account::<TokenAccount>::try_from(&ctx.accounts.checker_deal_ta) {
                Ok(checker_deal_ta) => {
                    check_ta(
                        &checker_deal_ta,
                        ctx.accounts.deal_mint.to_account_info().key,
                        ctx.accounts.checker.key,
                    )
                    .map_err(|_| InvalidAccount::CheckerDealTokenAccount)?;
                }
                Err(_) => {
                    init_ata(
                        &ctx.accounts.payer,
                        &ctx.accounts.deal_mint.to_account_info(),
                        &ctx.accounts.checker,
                        &ctx.accounts.checker_deal_ta,
                        &ctx.accounts.token_program,
                    )?;
                }
            };
        }

        if let Some(Bond { mint, .. }) = ctx.accounts.deal_state.client_bond.as_ref() {
            if !cmp_pubkeys(&mint, &ctx.accounts.client_bond_mint.key) {
                return Err(InvalidAccount::ClientBondMint)?;
            }

            let deal_state_client_bond_ta =
                Account::<TokenAccount>::try_from(&ctx.accounts.deal_state_client_bond_ta)?;
            match Account::<TokenAccount>::try_from(&ctx.accounts.client_bond_ta) {
                Ok(client_bond_ta) => {
                    check_ta(
                        &client_bond_ta,
                        &ctx.accounts.client_bond_mint.key(),
                        ctx.accounts.client.key,
                    )
                    .map_err(|_| InvalidAccount::ClientBondTokenAccount)?;
                    check_ta(
                        &deal_state_client_bond_ta,
                        &ctx.accounts.client_bond_mint.key(),
                        ctx.accounts.deal_state.to_account_info().key,
                    )
                    .map_err(|_| InvalidAccount::DealStateClientBondTokenAccount)?;
                }
                Err(_) => {
                    init_ata(
                        &ctx.accounts.payer,
                        &ctx.accounts.client_bond_mint.to_account_info(),
                        &ctx.accounts.client,
                        &ctx.accounts.client_bond_ta,
                        &ctx.accounts.token_program,
                    )
                    .map_err(|_| InvalidAccount::ClientBondTokenAccount)?;
                }
            };
        };

        if let Some(Bond { mint, .. }) = ctx.accounts.deal_state.executor_bond.as_ref() {
            if !cmp_pubkeys(&mint, &ctx.accounts.executor_bond_mint.key) {
                return Err(InvalidAccount::ExecutorBondMint)?;
            }

            let deal_state_executor_bond_ta =
                Account::<TokenAccount>::try_from(&ctx.accounts.deal_state_executor_bond_ta)?;
            match Account::<TokenAccount>::try_from(&ctx.accounts.executor_bond_ta) {
                Ok(executor_bond_ta) => {
                    check_ta(
                        &executor_bond_ta,
                        &ctx.accounts.executor_bond_mint.key(),
                        ctx.accounts.executor.key,
                    )
                    .map_err(|_| InvalidAccount::ExecutorBondTokenAccount)?;
                    check_ta(
                        &deal_state_executor_bond_ta,
                        &ctx.accounts.executor_bond_mint.key(),
                        ctx.accounts.deal_state.to_account_info().key,
                    )
                    .map_err(|_| InvalidAccount::DealStateExecutorBondTokenAccount)?;
                }
                Err(_) => {
                    init_ata(
                        &ctx.accounts.payer,
                        &ctx.accounts.executor_bond_mint.to_account_info(),
                        &ctx.accounts.executor,
                        &ctx.accounts.executor_bond_ta,
                        &ctx.accounts.token_program,
                    )
                    .map_err(|_| InvalidAccount::ExecutorBondTokenAccount)?;
                }
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

    fn transfer_checker_fee(&self) -> Result<CheckerFeeTransfered> {
        if let Some(Checker { checker_fee, .. }) = self.deal_state.checker {
            token::transfer(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    Transfer {
                        from: self.deal_state_deal_ta.to_account_info(),
                        to: self.checker_deal_ta.to_account_info(),
                        authority: self.deal_state.to_account_info(),
                    },
                    &[&self.deal_state.seeds()[..]],
                ),
                checker_fee,
            )?;
        };
        Ok(CheckerFeeTransfered)
    }

    fn transfer_deposit(&self) -> Result<DepositTransfered> {
        // let checker_fee = if let Some(Checker{checker_fee,..}) = self.deal_state.checker {checker_fee} else {0};
        token::transfer(
            CpiContext::new_with_signer(
                self.token_program.to_account_info(),
                Transfer {
                    from: self.deal_state_deal_ta.to_account_info(),
                    to: self.client_deal_ta.to_account_info(),
                    authority: self.deal_state.to_account_info(),
                },
                &[&self.deal_state.seeds()[..]],
            ),
            self.deal_state.amount,
        )?;
        Ok(DepositTransfered)
    }

    fn transfer_bonds(&mut self, initializer: Initializer) -> Result<BondsTransfered> {
        if let Some(Bond { amount, .. }) = self.deal_state.client_bond.as_ref() {
            if let Initializer::Executor = initializer {
                return Err(ErrorCodes::DealWithClientBond.into());
            }
            if *amount > 0 {
                anchor_spl::token::transfer(
                    CpiContext::new(
                        self.token_program.to_account_info(),
                        token::Transfer {
                            from: self.deal_state_client_bond_ta.to_account_info(),
                            to: self.client_bond_ta.to_account_info(),
                            authority: self.deal_state.to_account_info(),
                        },
                    )
                    .with_signer(&[&self.deal_state.seeds()[..]]),
                    *amount,
                )?;
            }
        }
        if let Some(Bond { amount, .. }) = self.deal_state.executor_bond.as_ref() {
            if let Initializer::Client = initializer {
                return Err(ErrorCodes::DealWithExecutorBond.into());
            }
            if *amount > 0 {
                anchor_spl::token::transfer(
                    CpiContext::new(
                        self.token_program.to_account_info(),
                        token::Transfer {
                            from: self.deal_state_client_bond_ta.to_account_info(),
                            to: self.executor_bond_ta.to_account_info(),
                            authority: self.deal_state.to_account_info(),
                        },
                    )
                    .with_signer(&[&self.deal_state.seeds()[..]]),
                    *amount,
                )?;
            }
        }

        if self.deal_state.client_bond.is_some() {
            self.close_deal_state_ta(&self.deal_state_client_bond_ta.clone())?;
        }
        if self.deal_state.executor_bond.is_some()
            && !cmp_pubkeys(
                self.deal_state_client_bond_ta.key,
                self.deal_state_executor_bond_ta.key,
            )
        {
            self.close_deal_state_ta(&self.deal_state_executor_bond_ta.clone())?;
        }

        Ok(BondsTransfered)
    }

    fn close_deal_state_ta(&self, token_account: &AccountInfo<'info>) -> Result<AccountClosed> {
        token::close_account(CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            CloseAccount {
                account: token_account.clone(),
                destination: self.service_fee.to_account_info(),
                authority: self.deal_state.to_account_info(),
            },
            &[&self.deal_state.seeds()[..]],
        ))?;
        Ok(AccountClosed)
    }
}

#[access_control(Cancel::check_accounts(&ctx))]
pub fn handle(ctx: Context<Cancel>) -> Result<()> {
    let initializer = Initializer::try_from(&*ctx.accounts)?;
    let deadline_checked = ctx.accounts.check_deadline()?;
    let checker_fee_transfered = ctx.accounts.transfer_checker_fee()?;
    let deposit_transfered = ctx.accounts.transfer_deposit()?;
    let bonds_transfered = ctx.accounts.transfer_bonds(initializer)?;

    let deal_state_deal_ta_closed = if ctx.accounts.deal_state_deal_ta.to_account_info().lamports()
        == 0
    {
        AccountClosed
    } else {
        ctx.accounts
            .close_deal_state_ta(AsRef::<AccountInfo>::as_ref(&*ctx.accounts.deal_state_deal_ta))?
    };

    Checklist {
        deadline_checked,
        checker_fee_transfered,
        deposit_transfered,
        bonds_transfered,
        deal_state_deal_ta_closed,
    };

    Ok(())
}
