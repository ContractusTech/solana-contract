use anchor_lang::prelude::*;

use anchor_spl::{token::{
    self, Mint, TokenAccount, Transfer, Token,
}, token_interface::spl_token_2022::cmp_pubkeys, associated_token::AssociatedToken};

use crate::{
    state::DealState, 
    utils::{PaymentTransfered, DealStateUpdated}};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PartiallyPayArgs {
    pub amount: u64,
}

#[derive(Accounts)]
#[instruction(args: PartiallyPayArgs)]
pub struct PartiallyPay<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        signer, 
        constraint=cmp_pubkeys(&deal_state.client_key, client.key),
        constraint = !cmp_pubkeys(executor.to_account_info().key, client.to_account_info().key)
    )]
    pub client: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(constraint=cmp_pubkeys(&deal_state.executor_key, executor.key))]
    pub executor: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub payer: AccountInfo<'info>,

    #[account(address=deal_state.deal_token_mint)]
    pub deal_mint: Box<Account<'info, Mint>>,
    
    #[account(mut,
        associated_token::mint = deal_mint,
        associated_token::authority = client,
    )]
    pub client_deal_ta: Box<Account<'info, TokenAccount>>,
    #[account(init_if_needed, payer = payer,
        associated_token::mint = deal_mint,
        associated_token::authority = executor,
    )]
    pub executor_deal_ta: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub deal_state: Box<Account<'info, DealState>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}


#[allow(dead_code)]
struct Checklist {
    pub deal_state_updated: DealStateUpdated,
    pub payment_transfered: PaymentTransfered
}

impl<'info> PartiallyPay<'info> {
    fn update_deal_state(&mut self, amount: u64) -> DealStateUpdated {
        self.deal_state.paid_amount += amount;

        DealStateUpdated
    }
    
    fn transfer_payment(&self, amount: u64) -> Result<PaymentTransfered> {
        let cpi_accounts = Transfer {
            from: self.client_deal_ta.to_account_info(),
            to: self.executor_deal_ta.to_account_info(),
            authority: self.client.to_account_info(),
        };
        token::transfer(CpiContext::new(self.token_program.to_account_info(), cpi_accounts), amount)?;

        Ok(PaymentTransfered)
    }
}

pub fn handle(ctx: Context<PartiallyPay>, args: PartiallyPayArgs) -> Result<()> {
    let deal_state_updated = ctx.accounts.update_deal_state(args.amount);
    let payment_transfered = ctx.accounts.transfer_payment(args.amount)?;
    
    Checklist {
        payment_transfered,
        deal_state_updated
    };

    Ok(())
}
