use anchor_lang::prelude::*;
use std::convert::Into;

mod constants;
mod errors;
mod instructions;
mod state;

use instructions::*;

declare_id!("9kpWdyR2qtNT21MhLRTBbT21v5thz9hhB3zaPUhr6tbE");

#[program]
pub mod deal_contract {
    use super::*;

    pub fn initialize_with_checker(
        ctx: Context<Initialize>,
        id: Vec<u8>,
        amount: u64,
        service_fee: u64,
        checker_fee: u64,
        holder_mode: bool,
    ) -> Result<()> {
        instructions::initialize_with_checker::handle(
            ctx,
            id,
            amount,
            service_fee,
            checker_fee,
            holder_mode,
        )
    }

    pub fn initialize_with_bond(
        ctx: Context<InitializeBond>,
        id: Vec<u8>,
        amount: u64,
        client_bond_amount: u64,
        executor_bond_amount: u64,
        service_fee: u64,
        deadline_ts: i64,
        holder_mode: bool,
    ) -> Result<()> {
        instructions::initialize_with_bond::handle(
            ctx,
            id,
            amount,
            client_bond_amount,
            executor_bond_amount,
            service_fee,
            deadline_ts,
            holder_mode,
        )
    }

    pub fn finish(ctx: Context<Finish>, id: Vec<u8>) -> Result<()> {
        instructions::finish::handle(ctx, id)
    }

    pub fn finish_with_bond(ctx: Context<FinishWithBond>, id: Vec<u8>) -> Result<()> {
        instructions::finish_with_bond::handle(ctx, id)
    }

    pub fn cancel(ctx: Context<Cancel>, id: Vec<u8>) -> Result<()> {
        instructions::cancel::handle(ctx, id)
    }

    pub fn cancel_with_bond(ctx: Context<CancelWithBond>, id: Vec<u8>) -> Result<()> {
        instructions::cancel_with_bond::handle(ctx, id)
    }

    pub fn update_checker(ctx: Context<UpdateChecker>, id: Vec<u8>) -> Result<()> {
        // TODO: -
        instructions::update_checker::handle(ctx, id)
    }
}
