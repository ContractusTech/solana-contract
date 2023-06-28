pub use anchor_lang::prelude::*;

#[account]
pub struct DealState {
    pub is_started: bool,
    pub with_bond: bool,

    pub client_key: Pubkey,
    pub client_token_account_key: Pubkey,
    pub client_bond_token_account_key: Pubkey,
    pub client_bond_deposit_key: Pubkey,

    pub executor_key: Pubkey,
    pub executor_token_account_key: Pubkey,
    pub executor_bond_token_account_key: Pubkey,
    pub executor_bond_deposit_key: Pubkey,

    pub checker_key: Pubkey,
    pub checker_token_account_key: Pubkey,
    pub deposit_key: Pubkey,
    pub holder_mode_deposit_key: Pubkey,

    pub authority_key: Pubkey,
    // pub service_key: Pubkey,
    pub amount: u64,
    pub client_bond_amount: u64,
    pub executor_bond_amount: u64,
    pub checker_fee: u64,

    pub deadline_ts: i64,

    pub bump: u8,
    pub deposit_bump: u8,
    pub authority_bump: u8,
    pub holder_deposit_bump: u8,
    pub client_bond_deposit_bump: u8,
    pub executor_bond_deposit_bump: u8,
}

impl DealState {
    pub fn space() -> usize {
        // 2 + (14 * 32) + (5 * 8) + 6 + 8
        8 + std::mem::size_of::<Self>() // 8 is for anchor discriminator
    }
}
