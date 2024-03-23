use anchor_lang::prelude::*;
use std::convert::Into;

mod constants;
mod errors;
mod instructions;
mod state;
mod utils;

use instructions::*;

declare_id!("GKNkN4uDJWmidEC9h5Q9GQXNg48Go6q5bdnkDj6bSopz");

#[program]
pub mod deal_contract {

    use super::*;

    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        instructions::initialize::handle(ctx, args)
    }

    pub fn finish(ctx: Context<Finish>) -> Result<()> {
        instructions::finish::handle(ctx)
    }

    pub fn cancel(ctx: Context<Cancel>) -> Result<()> {
        instructions::cancel::handle(ctx)
    }

    pub fn update_checker(ctx: Context<UpdateChecker>, new_checker_fee: u64) -> Result<()> {
        instructions::update_checker::handle(ctx, new_checker_fee)
    }

    pub fn partially_pay(ctx: Context<PartiallyPay>, args: PartiallyPayArgs) -> Result<()> {
        instructions::partially_pay::handle(ctx, args)
    }
}
