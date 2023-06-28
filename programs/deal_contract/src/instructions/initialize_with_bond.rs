use anchor_lang::prelude::*;

use anchor_spl::token::{
    self, spl_token::instruction::AuthorityType, Mint, SetAuthority, TokenAccount, Transfer, Token,
};

use crate::{constants::*, errors::ErrorCodes, state::{DealState, DealStateType}};

#[derive(Accounts)]
#[instruction(id: Vec<u8>, amount: u64, client_bond_amount: u64, executor_bond_amount: u64, service_fee: u64, deadline_ts: i64, holder_mode: bool)]
pub struct InitializeBond<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut, 
        signer, 
        constraint = *executor.to_account_info().key != *client.to_account_info().key
    )]
    pub client: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub executor: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub payer: AccountInfo<'info>,
    #[account(
        mut,
        constraint = service_fee_account.mint.key() == client_token_account.mint.key()
    )]
    pub service_fee_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = executor_token_account.mint.key() == client_token_account.mint.key(),
        constraint = executor_token_account.owner.key() == executor.key()
    )]
    pub executor_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = client_token_account.owner.key() == client.key(),
        constraint = executor_token_account.mint.key() == mint.key(),
        constraint = client_token_account.amount >= (amount + service_fee),
        constraint = (holder_mode && client_token_account.mint.key() == client_service_token_account.mint.key() && client_token_account.amount > (amount + HOLDER_MODE_AMOUNT)) 
                        || client_token_account.mint.key() != client_service_token_account.mint.key()
    )]
    pub client_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        constraint = client_service_token_account.owner.key() == *client.to_account_info().key,
        constraint = client_service_token_account.mint.key() == SERVICE_TOKEN_ADDRESS_MINT,
        constraint = (holder_mode && client_service_token_account.amount >= HOLDER_MODE_AMOUNT) || !holder_mode
    )]
    pub client_service_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = client_bond_account.owner == *client.to_account_info().key,
        constraint = client_bond_account.mint == client_bond_mint.key(),
        constraint = 
            (client_bond_account.mint == mint.key() 
                && client_bond_account.amount >= (client_bond_amount + service_fee + amount))
            || (client_bond_account.mint != mint.key() 
                && client_bond_account.amount >= client_bond_amount)
    )]
    pub client_bond_account: Box<Account<'info, TokenAccount>>,
    pub client_bond_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        constraint = executor_bond_account.owner.key() == *executor.to_account_info().key,
        constraint = executor_bond_account.mint.key() == executor_bond_mint.key(),
        constraint = executor_bond_account.amount >= executor_bond_amount
    )]
    pub executor_bond_account: Box<Account<'info, TokenAccount>>,
    pub executor_bond_mint: Box<Account<'info, Mint>>,

    pub mint: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = payer,
        token::mint = mint,
        token::authority = payer,
    )]
    pub deposit_account: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = payer,
        token::mint = client_bond_mint,
        token::authority = payer,
    )]
    pub deposit_client_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = payer,
        token::mint = executor_bond_mint,
        token::authority = payer,
    )]
    pub deposit_executor_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = payer,
        token::mint = holder_mint,
        token::authority = payer,
    )]
    pub holder_deposit_account: Box<Account<'info, TokenAccount>>,
    pub holder_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        seeds = [&id, DEAL_STATE_SEED, client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
        payer = payer, 
        space = DealState::space()
    )]
    pub deal_state: Box<Account<'info, DealState>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

// Initialize Bond
impl<'info> InitializeBond<'info> {
    fn into_transfer_to_pda_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.client_token_account.to_account_info(),
            to: self.deposit_account.to_account_info(),
            authority: self.client.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_service_account(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.client_token_account.to_account_info(),
            to: self.service_fee_account.to_account_info(),
            authority: self.client.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_holder_mode_account(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.client_token_account.to_account_info(),
            to: self.holder_deposit_account.to_account_info(),
            authority: self.client.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_executor_bond_account(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.executor_bond_account.to_account_info(),
            to: self.deposit_executor_bond_account.to_account_info(),
            authority: self.executor.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_transfer_to_client_bond_account(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.client_bond_account.to_account_info(),
            to: self.deposit_client_bond_account.to_account_info(),
            authority: self.client.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.deposit_account.to_account_info(),
            current_authority: self.payer.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_set_authority_holder_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.holder_deposit_account.to_account_info(),
            current_authority: self.payer.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_set_authority_executor_bond_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.deposit_executor_bond_account.to_account_info(),
            current_authority: self.payer.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    fn into_set_authority_client_bond_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.deposit_client_bond_account.to_account_info(),
            current_authority: self.payer.clone(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }
}

pub fn handle(
    ctx: Context<InitializeBond>,
    _id: Vec<u8>,
    amount: u64,
    client_bond_amount: u64,
    executor_bond_amount: u64,
    service_fee: u64,
    deadline_ts: i64,
    holder_mode: bool,
) -> Result<()> {
    let current_ts = Clock::get()?.unix_timestamp;
    if deadline_ts < current_ts {
        return Err(ErrorCodes::DeadlineExpired.into());
    }

    if amount == 0 {
        return Err(ErrorCodes::AmountTooLow.into());
    }

    if holder_mode
        && ctx.accounts.client_service_token_account.amount < HOLDER_MODE_AMOUNT
        && ctx.accounts.client_service_token_account.mint != SERVICE_TOKEN_ADDRESS_MINT
        && ctx.accounts.client_token_account.mint != SERVICE_TOKEN_ADDRESS_MINT
    {
        return Err(ErrorCodes::HolderModeUnavailable.into());
    }
    if !holder_mode && service_fee == 0 {
        return Err(ErrorCodes::FeeIsTooLow.into());
    }

    **ctx.accounts.deal_state = DealState {
        client_key: *ctx.accounts.client.key,
        executor_key: *ctx.accounts.executor.to_account_info().key,
        deposit_key: *ctx.accounts.deposit_account.to_account_info().key,

        holder_mode_deposit_key:
        *ctx.accounts.holder_deposit_account.to_account_info().key,
        bump: *ctx.bumps.get("deal_state").unwrap(),
        deposit_bump: *ctx.bumps.get("deposit_account").unwrap(),
        holder_deposit_bump:
        *ctx.bumps.get("holder_deposit_account").unwrap(),
        client_bond_deposit_bump:
        *ctx.bumps.get("deposit_client_bond_account").unwrap(),
        executor_bond_deposit_bump:
        *ctx.bumps.get("deposit_executor_bond_account").unwrap(),

        amount,

        deadline_ts,

        _type: DealStateType::WithBond { 
            client_bond_deposit_key:
            *ctx.accounts.deposit_client_bond_account.to_account_info().key,
            executor_bond_deposit_key:
            *ctx.accounts.deposit_executor_bond_account.to_account_info().key,
            client_bond_amount,
            executor_bond_amount,

        }
    };

    token::set_authority(
        ctx.accounts.into_set_authority_context(),
        AuthorityType::AccountOwner,
        Some(*ctx.accounts.deal_state.to_account_info().key),
    )?;

    token::set_authority(
        ctx.accounts.into_set_authority_holder_context(),
        AuthorityType::AccountOwner,
        Some(*ctx.accounts.deal_state.to_account_info().key),
    )?;

    token::transfer(ctx.accounts.into_transfer_to_pda_context(), amount)?;

    if holder_mode {
        token::transfer(ctx.accounts.into_transfer_to_holder_mode_account(), HOLDER_MODE_AMOUNT)?;
    } else if service_fee > 0 {
        token::transfer(ctx.accounts.into_transfer_to_service_account(), service_fee)?;
    }

    if client_bond_amount > 0 {
        token::set_authority(
            ctx.accounts.into_set_authority_client_bond_context(),
            AuthorityType::AccountOwner,
            Some(*ctx.accounts.deal_state.to_account_info().key),
        )?;

        token::transfer(ctx.accounts.into_transfer_to_client_bond_account(), client_bond_amount)?;
    }

    if executor_bond_amount > 0 {
        token::set_authority(
            ctx.accounts.into_set_authority_executor_bond_context(),
            AuthorityType::AccountOwner,
            Some(*ctx.accounts.deal_state.to_account_info().key),
        )?;

        token::transfer(
            ctx.accounts.into_transfer_to_executor_bond_account(),
            executor_bond_amount,
        )?;
    }

    Ok(())
}
