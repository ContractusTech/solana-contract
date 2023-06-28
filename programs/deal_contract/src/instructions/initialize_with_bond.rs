use anchor_lang::prelude::*;

use anchor_spl::token::{
    self, spl_token::instruction::AuthorityType, Mint, SetAuthority, TokenAccount, Transfer,
};

use crate::{constants::*, errors::ErrorCode, state::DealState};

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
        constraint = client_bond_account.owner.key() == *client.to_account_info().key,
        constraint = client_bond_account.mint.key() == client_bond_mint.key(),
        constraint = (client_bond_account.mint == mint.key() && client_bond_account.amount >= (client_bond_amount + service_fee + amount))
            || (client_bond_account.mint.key() != mint.key() && client_bond_account.amount >= client_bond_amount)
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
        seeds = [&id, b"deposit".as_ref(), client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
        payer = payer,
        token::mint = mint,
        token::authority = payer,
    )]
    pub deposit_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        seeds = [&id, b"auth".as_ref(), client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
    )]
    pub authority: AccountInfo<'info>,
    #[account(
        init_if_needed,
        seeds = [&id, b"deposit_bond_client".as_ref(), client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
        payer = payer,
        token::mint = client_bond_mint,
        token::authority = payer,
    )]
    pub deposit_client_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        seeds = [&id, b"deposit_bond_executor".as_ref(), client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
        payer = payer,
        token::mint = executor_bond_mint,
        token::authority = payer,
    )]
    pub deposit_executor_bond_account: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        seeds = [&id, b"holder_deposit".as_ref(), client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
        payer = payer,
        token::mint = holder_mint,
        token::authority = payer,
    )]
    pub holder_deposit_account: Box<Account<'info, TokenAccount>>,
    pub holder_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        seeds = [&id, b"state".as_ref(), client.to_account_info().key.as_ref(), executor.to_account_info().key.as_ref()],
        bump,
        payer = payer, 
        space = DealState::space()
    )]
    pub deal_state: Box<Account<'info, DealState>>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub system_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub token_program: AccountInfo<'info>,
}

// Initialize Bond
impl<'info> InitializeBond<'info> {
    fn into_transfer_to_pda_context(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.client_token_account.to_account_info(),
            to: self.deposit_account.to_account_info(),
            authority: self.client.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_service_account(&self) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.client_token_account.to_account_info(),
            to: self.service_fee_account.to_account_info(),
            authority: self.client.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_holder_mode_account(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.client_token_account.to_account_info(),
            to: self.holder_deposit_account.to_account_info(),
            authority: self.client.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_executor_bond_account(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.executor_bond_account.to_account_info(),
            to: self.deposit_executor_bond_account.to_account_info(),
            authority: self.executor.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_transfer_to_client_bond_account(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        let cpi_accounts = Transfer {
            from: self.client_bond_account.to_account_info(),
            to: self.deposit_client_bond_account.to_account_info(),
            authority: self.client.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_set_authority_context(&self) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.deposit_account.to_account_info(),
            current_authority: self.payer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_set_authority_holder_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.holder_deposit_account.to_account_info(),
            current_authority: self.payer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_set_authority_executor_bond_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.deposit_executor_bond_account.to_account_info(),
            current_authority: self.payer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }

    fn into_set_authority_client_bond_context(
        &self,
    ) -> CpiContext<'_, '_, '_, 'info, SetAuthority<'info>> {
        let cpi_accounts = SetAuthority {
            account_or_mint: self.deposit_client_bond_account.to_account_info(),
            current_authority: self.payer.clone(),
        };
        CpiContext::new(self.token_program.clone(), cpi_accounts)
    }
}

pub fn handle(
    _ctx: Context<InitializeBond>,
    _id: Vec<u8>,
    amount: u64,
    client_bond_amount: u64,
    executor_bond_amount: u64,
    service_fee: u64,
    deadline_ts: i64,
    holder_mode: bool,
) -> Result<()> {
    if _ctx.accounts.deal_state.is_started {
        return Err(ErrorCode::AlreadyStarted.into());
    }

    let clock = Clock::get()?;
    let current_ts = clock.unix_timestamp;
    if deadline_ts < current_ts {
        return Err(ErrorCode::DeadlineExpired.into());
    }

    if amount == 0 {
        return Err(ErrorCode::AmountTooLow.into());
    }

    let service_token_mint: Pubkey = SERVICE_TOKEN_ADDRESS_MINT;

    if holder_mode
        && _ctx.accounts.client_service_token_account.amount < HOLDER_MODE_AMOUNT
        && _ctx.accounts.client_service_token_account.mint != service_token_mint
        && _ctx.accounts.client_token_account.mint != service_token_mint
    {
        return Err(ErrorCode::HolderModeUnavailable.into());
    }
    if !holder_mode && service_fee == 0 {
        return Err(ErrorCode::FeeIsTooLow.into());
    }

    _ctx.accounts.deal_state.is_started = true;

    _ctx.accounts.deal_state.client_key = *_ctx.accounts.client.key;
    _ctx.accounts.deal_state.executor_key = *_ctx.accounts.executor.to_account_info().key;
    _ctx.accounts.deal_state.deposit_key = *_ctx.accounts.deposit_account.to_account_info().key;
    _ctx.accounts.deal_state.authority_key = *_ctx.accounts.authority.to_account_info().key;

    _ctx.accounts.deal_state.client_token_account_key =
        *_ctx.accounts.client_token_account.to_account_info().key;
    _ctx.accounts.deal_state.client_bond_deposit_key =
        *_ctx.accounts.deposit_client_bond_account.to_account_info().key;
    _ctx.accounts.deal_state.client_bond_token_account_key =
        *_ctx.accounts.client_bond_account.to_account_info().key;

    _ctx.accounts.deal_state.executor_token_account_key =
        *_ctx.accounts.executor_token_account.to_account_info().key;
    _ctx.accounts.deal_state.executor_bond_deposit_key =
        *_ctx.accounts.deposit_executor_bond_account.to_account_info().key;
    _ctx.accounts.deal_state.executor_bond_token_account_key =
        *_ctx.accounts.executor_bond_account.to_account_info().key;

    _ctx.accounts.deal_state.holder_mode_deposit_key =
        *_ctx.accounts.holder_deposit_account.to_account_info().key;

    // ctx.accounts.deal_state.service_key = Pubkey::from_str(SERVICE_ACCOUNT_ADDRESS).unwrap();

    _ctx.accounts.deal_state.bump = *_ctx.bumps.get("deal_state").unwrap();
    _ctx.accounts.deal_state.deposit_bump = *_ctx.bumps.get("deposit_account").unwrap();
    _ctx.accounts.deal_state.authority_bump = *_ctx.bumps.get("authority").unwrap();
    _ctx.accounts.deal_state.holder_deposit_bump =
        *_ctx.bumps.get("holder_deposit_account").unwrap();
    _ctx.accounts.deal_state.client_bond_deposit_bump =
        *_ctx.bumps.get("deposit_client_bond_account").unwrap();
    _ctx.accounts.deal_state.executor_bond_deposit_bump =
        *_ctx.bumps.get("deposit_executor_bond_account").unwrap();

    _ctx.accounts.deal_state.amount = amount;
    _ctx.accounts.deal_state.client_bond_amount = client_bond_amount;
    _ctx.accounts.deal_state.executor_bond_amount = executor_bond_amount;

    _ctx.accounts.deal_state.with_bond = true;
    _ctx.accounts.deal_state.deadline_ts = deadline_ts;

    token::set_authority(
        _ctx.accounts.into_set_authority_context(),
        AuthorityType::AccountOwner,
        Some(*_ctx.accounts.authority.to_account_info().key),
    )?;

    token::set_authority(
        _ctx.accounts.into_set_authority_holder_context(),
        AuthorityType::AccountOwner,
        Some(*_ctx.accounts.authority.to_account_info().key),
    )?;

    token::transfer(_ctx.accounts.into_transfer_to_pda_context(), amount)?;

    if holder_mode {
        token::transfer(_ctx.accounts.into_transfer_to_holder_mode_account(), HOLDER_MODE_AMOUNT)?;
    } else if service_fee > 0 {
        token::transfer(_ctx.accounts.into_transfer_to_service_account(), service_fee)?;
    }

    if client_bond_amount > 0 {
        token::set_authority(
            _ctx.accounts.into_set_authority_client_bond_context(),
            AuthorityType::AccountOwner,
            Some(*_ctx.accounts.authority.to_account_info().key),
        )?;

        token::transfer(_ctx.accounts.into_transfer_to_client_bond_account(), client_bond_amount)?;
    }

    if executor_bond_amount > 0 {
        token::set_authority(
            _ctx.accounts.into_set_authority_executor_bond_context(),
            AuthorityType::AccountOwner,
            Some(*_ctx.accounts.authority.to_account_info().key),
        )?;

        token::transfer(
            _ctx.accounts.into_transfer_to_executor_bond_account(),
            executor_bond_amount,
        )?;
    }

    Ok(())
}
