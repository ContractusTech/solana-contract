use crate::{
    constants::*,
    errors::ErrorCodes,
    state::{DealState, DealStateType},
};
use anchor_lang::prelude::*;
use anchor_spl::{token::{
    self, spl_token::instruction::AuthorityType, Mint, SetAuthority, TokenAccount, Transfer, Token,
}, token_interface::spl_token_2022::cmp_pubkeys};

#[derive(Accounts)]
#[instruction(id: Vec<u8>, amount: u64, service_fee: u64, checker_fee: u64, holder_mode: bool)]
pub struct Initialize<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        mut, 
        signer,
        constraint = *executor.to_account_info().key != *client.to_account_info().key 
        &&  *checker.to_account_info().key != *client.to_account_info().key
    )]
    pub client: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub executor: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub checker: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub payer: AccountInfo<'info>,
    #[account(
        mut,
        constraint = cmp_pubkeys(&service_fee_account.mint, &client_token_account.mint.key())
    )]
    pub service_fee_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = executor_token_account.mint == client_token_account.mint.key(),
        constraint = executor_token_account.owner == executor.key()
    )]
    pub executor_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = client_token_account.owner == client.key(),
        constraint = executor_token_account.mint == mint.key(),
        constraint = client_token_account.amount >= (amount + service_fee + checker_fee),
        constraint = (holder_mode && client_token_account.mint == SERVICE_TOKEN_ADDRESS_MINT 
            && client_token_account.amount > (amount + service_fee + checker_fee + HOLDER_MODE_AMOUNT)) 
            || !holder_mode
    )]
    pub client_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        constraint = client_service_token_account.owner == *client.to_account_info().key,
        constraint = client_service_token_account.mint == SERVICE_TOKEN_ADDRESS_MINT,
    )]
    pub client_service_token_account: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        constraint = checker_token_account.mint == mint.key(),
        constraint = checker_token_account.owner == checker.key()
    )]
    pub checker_token_account: Box<Account<'info, TokenAccount>>,
    pub mint: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = payer,
        token::mint = mint,
        token::authority = deal_state,
    )]
    pub deposit_account: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer = payer,
        token::mint = holder_mint,
        token::authority = deal_state,
    )]
    pub holder_deposit_account: Box<Account<'info, TokenAccount>>,
    #[account(address = HOLDER_MINT)]
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

// Initialize
impl<'info> Initialize<'info> {
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
}

pub fn handle(
    ctx: Context<Initialize>,
    _id: Vec<u8>,
    amount: u64,
    service_fee: u64,
    checker_fee: u64,
    holder_mode: bool,
) -> Result<()> {
    if amount == 0 {
        return Err(ErrorCodes::AmountTooLow.into());
    }

    let service_token_mint: Pubkey = SERVICE_TOKEN_ADDRESS_MINT;

    if holder_mode
        && ctx.accounts.client_service_token_account.amount < HOLDER_MODE_AMOUNT
        && ctx.accounts.client_service_token_account.mint != service_token_mint
        && ctx.accounts.client_token_account.mint != service_token_mint
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

        deadline_ts: 0,

        _type: DealStateType::WithChecker {
            checker_fee, 
            checker_key: *ctx.accounts.checker.key
            
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

    token::transfer(ctx.accounts.into_transfer_to_pda_context(), amount + checker_fee)?;

    if holder_mode {
        token::transfer(ctx.accounts.into_transfer_to_holder_mode_account(), HOLDER_MODE_AMOUNT)?;
    } else if service_fee > 0 {
        token::transfer(ctx.accounts.into_transfer_to_service_account(), service_fee)?;
    }

    Ok(())
}
