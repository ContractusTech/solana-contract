use anchor_lang::prelude::*;
use anchor_spl::{token::TokenAccount, token_interface::spl_token_2022::cmp_pubkeys};

use crate::errors::ErrorCodes;

pub(crate) mod checklist;

// pub(crate) struct SignaturesChecked;
pub(crate) struct DealStateCreated;
pub(crate) struct DeadlineChecked;
pub(crate) struct DealAmountChecked;

pub(crate) struct CheckerFeeTransfered;
pub(crate) struct DepositTransfered;
pub(crate) struct BondsTransfered;
pub(crate) struct HolderModeHandled;

pub(crate) struct PaymentTransfered;
pub(crate) struct AdvancePaymentTransfered;
// pub(crate) struct PaymentReturned;

// pub(crate) struct CheckerAccountsChecked;
// pub(crate) struct BondAccountsChecked;

pub(crate) struct AccountClosed;

pub fn init_ata<'a, 'info>(
    payer: &'a AccountInfo<'info>,
    mint: &'a AccountInfo<'info>,
    authority: &'a AccountInfo<'info>,
    ata: &'a AccountInfo<'info>,
    token_program: &'a AccountInfo<'info>,
) -> Result<()> {
    solana_program::program::invoke(
        &spl_associated_token_account::instruction::create_associated_token_account(
            payer.key,
            authority.key,
            mint.key,
            token_program.key,
        ),
        &[
            payer.clone(),
            mint.clone(),
            authority.clone(),
            ata.clone(),
            token_program.clone(),
        ],
    )?;

    Ok(())
}

pub fn check_ta<'info>(
    token_account: &Account<'info, TokenAccount>,
    expected_mint: &Pubkey,
    expected_owner: &Pubkey,
) -> Result<()> {
    if !cmp_pubkeys(&token_account.mint, &expected_mint) {
        return Err(ErrorCodes::InvalidMint.into());
    };
    if !cmp_pubkeys(&token_account.owner, &expected_owner) {
        return Err(ErrorCodes::InvalidOwner.into());
    };
    Ok(())
}
