use anchor_lang::prelude::*;

use anchor_spl::{token::{
    self, Mint, TokenAccount, Transfer, Token,
}, token_interface::spl_token_2022::cmp_pubkeys, associated_token::AssociatedToken};

use crate::{constants::*, 
    errors::{ErrorCodes, InvalidAccount}, 
    state::{DealState, Bond, Checker }, 
    utils::{DeadlineChecked, DealStateCreated, BondsTransfered, HolderModeHandled, DepositTransfered, CheckerFeeTransfered, DealAmountChecked}};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeArgs {
    pub id: [u8; 16],
    pub deal_amount: u64,
    pub service_fee: u64,
    pub deadline_ts: Option<i64>,
    pub holder_mode: bool,
    pub client_bond: Option<u64>,
    pub executor_bond: Option<u64>,
    pub checker_fee: Option<u64>,
}

#[derive(Accounts)]
#[instruction(args: InitializeArgs)]
pub struct Initialize<'info> {
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(
        signer, 
        constraint = !cmp_pubkeys(executor.to_account_info().key, client.to_account_info().key)
    )]
    pub client: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(signer)]
    pub executor: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub payer: AccountInfo<'info>,
    /// CHECK: This is not dangerous because we don't read or write from this account
    #[account(mut, signer)]
    pub checker: AccountInfo<'info>,

    pub deal_mint: Box<Account<'info, Mint>>,
    pub client_bond_mint: Box<Account<'info, Mint>>,
    pub executor_bond_mint: Box<Account<'info, Mint>>,
    pub service_mint: Box<Account<'info, Mint>>,
    #[account(address = HOLDER_MINT )]
    pub holder_mint: Box<Account<'info, Mint>>,
    
    /// CHECK: by address
    #[account(address = SERVICE_FEE_OWNER)]
    pub service_fee_owner: AccountInfo<'info>,
    #[account(init_if_needed, payer = payer,
        associated_token::mint = service_mint,
        associated_token::authority = service_fee_owner,
    )]
    pub service_fee_ta: Box<Account<'info, TokenAccount>>,

    #[account(init_if_needed, payer = payer,
        associated_token::mint = service_mint,
        associated_token::authority = client,
    )]
    pub client_service_ta: Box<Account<'info, TokenAccount>>,
    
    #[account(init_if_needed, payer = payer,
        associated_token::mint = deal_mint,
        associated_token::authority = client,
    )]
    pub client_deal_ta: Box<Account<'info, TokenAccount>>,
    #[account(init_if_needed, payer = payer,
        associated_token::mint = deal_mint,
        associated_token::authority = executor,
    )]
    pub executor_deal_ta: Box<Account<'info, TokenAccount>>,
    #[account(init_if_needed, payer = payer,
        associated_token::mint = deal_mint,
        associated_token::authority = deal_state,
    )]
    pub deal_state_deal_ta: Box<Account<'info, TokenAccount>>,

    #[account(init_if_needed, payer = payer,
        associated_token::mint = client_bond_mint,
        associated_token::authority = client,
    )]
    pub client_bond_ta: Box<Account<'info, TokenAccount>>,
    #[account(init_if_needed, payer = payer,
        associated_token::mint = executor_bond_mint,
        associated_token::authority = executor,
    )]
    pub executor_bond_ta: Box<Account<'info, TokenAccount>>,
    #[account(init_if_needed, payer = payer,
        associated_token::mint = client_bond_mint,
        associated_token::authority = deal_state,
    )]
    pub deal_state_client_bond_ta: Box<Account<'info, TokenAccount>>,
    #[account(init_if_needed, payer = payer,
        associated_token::mint = executor_bond_mint,
        associated_token::authority = deal_state,
    )]
    pub deal_state_executor_bond_ta: Box<Account<'info, TokenAccount>>,

    /// CHECK: must be checked if args.holder_mode == true
    #[account(mut)]
    pub client_holder_ta: AccountInfo<'info>,
    /// CHECK: must be initialized if args.holder_mode == true
    #[account(mut)]
    pub deal_state_holder_ta: AccountInfo<'info>,

    #[account(init_if_needed,
        seeds = [&args.id, DEAL_STATE_SEED, client.key.as_ref(), executor.key.as_ref()],
        bump,
        payer = payer, 
        space = 8 + std::mem::size_of::<DealState>() // 8 is for anchor discriminator
    )]
    pub deal_state: Box<Account<'info, DealState>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

// impl<'info> TransferBond<'info> for InitializeNew<'info> {}
// impl<'info> TransferPayment<'info> for InitializeNew<'info> {}


#[allow(dead_code)]
struct Checklist {
    pub deadline_checked: DeadlineChecked,
    pub amount_checked: DealAmountChecked,

    pub deal_state_created: DealStateCreated,
    pub bonds_transfered: BondsTransfered,
    pub holder_mode_handled: HolderModeHandled,
    pub deposit_transfered: DepositTransfered,
    pub checker_fee_transfered: CheckerFeeTransfered
}

impl<'info> Initialize<'info> {
    fn check_deadline(&self) -> Result<DeadlineChecked> {
        if self.deal_state.deadline_expired() {
            return Err(ErrorCodes::DeadlineExpired.into())
        };
        Ok(DeadlineChecked)
    }

    fn check_deal_amount(&self, deal_amount: u64) -> Result<DealAmountChecked> {
        if deal_amount == 0 {
            return Err(ErrorCodes::AmountTooLow.into());
        }
        Ok(DealAmountChecked)
    }

    fn transfer_bonds(&self, client_bond: Option<u64>, executor_bond: Option<u64>) -> Result<BondsTransfered> {
        if let Some(amount) = client_bond.as_ref() {
            if *amount > 0 {
                anchor_spl::token::transfer(CpiContext::new(
                    self.token_program.to_account_info(),
                    token::Transfer {
                        from: self.client_bond_ta.to_account_info(),
                        to: self.deal_state_client_bond_ta.to_account_info(),
                        authority: self.client.to_account_info(),
                    },
                ), *amount)?;
            }
        }
        if let Some(amount) = executor_bond.as_ref() {
            if *amount > 0 {
                anchor_spl::token::transfer(CpiContext::new(
                    self.token_program.to_account_info(),
                    token::Transfer {
                        from: self.executor_bond_ta.to_account_info(),
                        to: self.deal_state_client_bond_ta.to_account_info(),
                        authority: self.executor.to_account_info(),
                    },
                ), *amount)?;
            }
        }
        Ok(BondsTransfered)
    }

    fn transfer_deposit(&self, amount: u64) -> Result<DepositTransfered> {
        anchor_spl::token::transfer(CpiContext::new(
            self.token_program.to_account_info(),
            token::Transfer {
                from: self.client_bond_ta.to_account_info(),
                to: self.deal_state_client_bond_ta.to_account_info(),
                authority: self.client.to_account_info(),
            },
        ), amount)?;
        Ok(DepositTransfered)
    }

    fn handle_service_fee(&self, holder_mode: bool, service_fee: u64) -> Result<HolderModeHandled> {
        if !holder_mode && service_fee == 0 {
            return Err(ErrorCodes::FeeIsTooLow.into());
        };

        if holder_mode {
            let client_holder_ta = Account::<'info, TokenAccount>::try_from(&self.client_holder_ta)?;

            require_keys_eq!(client_holder_ta.mint, HOLDER_MINT, InvalidAccount::ClientHolderTokenAccountMint);
            require_keys_eq!(client_holder_ta.owner, self.client.key(), InvalidAccount::ClientHolderTokenAccountOwner);

            msg!("client_holder_ta: {:?}", client_holder_ta);

            if self.client_deal_ta.mint != SERVICE_FEE_MINT { 
                return Err(ErrorCodes::HolderModeUnavailable.into()); 
            }
            
            //TODO add checks here 
            let cpi_accounts = Transfer {
                from: self.client_deal_ta.to_account_info(),
                to: self.deal_state_holder_ta.to_account_info(),
                authority: self.client.clone(),
            };
            token::transfer(CpiContext::new(self.token_program.to_account_info(), cpi_accounts), HOLDER_MODE_AMOUNT)?;
        } else if service_fee > 0 {
            let cpi_accounts = Transfer {
                from: self.client_deal_ta.to_account_info(),
                to: self.service_fee_ta.to_account_info(),
                authority: self.client.clone(),
            };
            token::transfer(CpiContext::new(self.token_program.to_account_info(), cpi_accounts), service_fee)?;        
        }
        
        Ok(HolderModeHandled)
    }
}

pub fn handle(ctx: Context<Initialize>,args: InitializeArgs) -> Result<()> {
    let deal_state_created = {
        **ctx.accounts.deal_state = DealState {
            id: args.id,
            client_key: *ctx.accounts.client.key,
            executor_key: *ctx.accounts.executor.to_account_info().key,

            bump: [*ctx.bumps.get("deal_state").unwrap()],
            client_bond: if let Some(amount) = args.client_bond { Some(Bond {mint: ctx.accounts.client_bond_mint.key(), amount}) } else { None },
            executor_bond: if let Some(amount) = args.executor_bond { Some(Bond {mint: ctx.accounts.executor_bond_mint.key(), amount}) } else { None },
            checker: if let Some(checker_fee) = args.checker_fee.as_ref() { 
                Some(Checker {checker_fee: *checker_fee, checker_key: ctx.accounts.checker.key()})
            } else { None},

            amount: args.deal_amount,

            deadline_ts: args.deadline_ts,
            deal_token_mint: ctx.accounts.deal_mint.to_account_info().key(),
            holder_mode: if args.holder_mode { Some(HOLDER_MODE_AMOUNT) } else { None },
        };
        DealStateCreated
    };

    let deadline_checked = ctx.accounts.check_deadline()?;
    
    let amount_checked = ctx.accounts.check_deal_amount(args.deal_amount)?;

    let holder_mode_handled = ctx.accounts.handle_service_fee(args.holder_mode, args.service_fee)?;
    
    let (deposit_transfered, checker_fee_transfered) = {
        ctx.accounts.transfer_deposit(args.deal_amount + if let Some(checker_fee) = args.checker_fee { checker_fee } else { 0 })?;
        (DepositTransfered, CheckerFeeTransfered)
    };

    let bonds_transfered = ctx.accounts.transfer_bonds(args.client_bond, args.executor_bond)?;

    Checklist {
        deadline_checked,
        amount_checked,
        checker_fee_transfered,
        deposit_transfered,
        deal_state_created,
        bonds_transfered,
        holder_mode_handled
    };

    Ok(())
}
