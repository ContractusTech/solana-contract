use crate::{constants::DEAL_STATE_SEED, errors::ErrorCodes};
pub use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Bond {
    pub mint: Pubkey,
    pub amount: u64,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Checker {
    pub checker_fee: u64,
    pub checker_key: Pubkey,
}

#[account]
pub struct DealState {
    pub id: [u8; 16],
    pub client_key: Pubkey,
    pub executor_key: Pubkey,

    pub deal_token_mint: Pubkey,

    pub client_bond: Option<Bond>,
    pub executor_bond: Option<Bond>,

    pub checker: Option<Checker>,

    pub holder_mode: Option<u64>,

    pub amount: u64,

    pub deadline_ts: Option<i64>,

    pub bump: [u8; 1],
}

impl DealState {
    pub fn id(&self) -> u128 {
        u128::from_le_bytes(self.id)
    }

    pub fn bump(&self) -> u8 {
        self.bump[0]
    }

    pub fn seeds<'a: 'b, 'b>(&'a self) -> [&'b [u8]; 5] {
        [
            &self.id[..],
            DEAL_STATE_SEED,
            self.client_key.as_ref(),
            self.executor_key.as_ref(),
            &self.bump,
        ]
    }

    pub fn client_bond(&self) -> Result<&Bond> {
        Ok(self.client_bond.as_ref().ok_or(ErrorCodes::NoClientBond)?)
    }
    pub fn client_bond_mut(&mut self) -> Result<&mut Bond> {
        Ok(self.client_bond.as_mut().ok_or(ErrorCodes::NoClientBond)?)
    }
    pub fn executor_bond(&self) -> Result<&Bond> {
        Ok(self.executor_bond.as_ref().ok_or(ErrorCodes::NoExecutorBond)?)
    }
    pub fn executor_bond_mut(&mut self) -> Result<&mut Bond> {
        Ok(self.executor_bond.as_mut().ok_or(ErrorCodes::NoExecutorBond)?)
    }
    pub fn with_checker(&self) -> Result<&Checker> {
        Ok(self.checker.as_ref().ok_or(ErrorCodes::DealStateNotWithChecker)?)
    }
    pub fn with_checker_mut(&mut self) -> Result<&mut Checker> {
        Ok(self.checker.as_mut().ok_or(ErrorCodes::DealStateNotWithChecker)?)
    }

    pub fn deadline_expired(&self) -> bool {
        match self.deadline_ts {
            Some(deadline_ts) => {
                let current_ts = Clock::get().expect("Failed to get Clock SysVar").unix_timestamp;
                if deadline_ts < current_ts {
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }
}
