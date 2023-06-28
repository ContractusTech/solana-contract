use crate::errors::ErrorCodes;
pub use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub enum DealStateType {
    WithBond {
        client_bond_deposit_key: Pubkey,
        executor_bond_deposit_key: Pubkey,
        client_bond_amount: u64,
        executor_bond_amount: u64,
    },
    WithChecker {
        checker_fee: u64,
        checker_key: Pubkey,
    },
}

#[account]
pub struct DealState {
    pub _type: DealStateType,
    pub client_key: Pubkey,

    pub executor_key: Pubkey,

    pub deposit_key: Pubkey,
    pub holder_mode_deposit_key: Pubkey,

    pub amount: u64,

    pub deadline_ts: i64,

    pub bump: u8,
    pub deposit_bump: u8,
    pub holder_deposit_bump: u8,
    pub client_bond_deposit_bump: u8,
    pub executor_bond_deposit_bump: u8,
}

impl DealState {
    pub fn space() -> usize {
        // 2 + (14 * 32) + (5 * 8) + 6 + 8
        8 + std::mem::size_of::<Self>() // 8 is for anchor discriminator
    }

    pub fn checker_fee(&self) -> Result<u64> {
        if let DealStateType::WithChecker { checker_fee, .. } = self._type {
            Ok(checker_fee)
        } else {
            Err(ErrorCodes::DealStateNotWithChecker.into())
        }
    }
    pub fn checker_key(&self) -> Result<&Pubkey> {
        if let DealStateType::WithChecker { checker_key, .. } = &self._type {
            Ok(checker_key)
        } else {
            Err(ErrorCodes::DealStateNotWithChecker.into())
        }
    }

    pub fn client_bond_deposit_key(&self) -> Result<&Pubkey> {
        if let DealStateType::WithBond {
            client_bond_deposit_key,
            ..
        } = &self._type
        {
            Ok(client_bond_deposit_key)
        } else {
            Err(ErrorCodes::DealStateNotWithBond.into())
        }
    }
    pub fn executor_bond_deposit_key(&self) -> Result<&Pubkey> {
        if let DealStateType::WithBond {
            executor_bond_deposit_key,
            ..
        } = &self._type
        {
            Ok(executor_bond_deposit_key)
        } else {
            Err(ErrorCodes::DealStateNotWithBond.into())
        }
    }
    pub fn client_bond_amount(&self) -> Result<u64> {
        if let DealStateType::WithBond {
            client_bond_amount, ..
        } = self._type
        {
            Ok(client_bond_amount)
        } else {
            Err(ErrorCodes::DealStateNotWithBond.into())
        }
    }
    pub fn executor_bond_amount(&self) -> Result<u64> {
        if let DealStateType::WithBond {
            executor_bond_amount,
            ..
        } = self._type
        {
            Ok(executor_bond_amount)
        } else {
            Err(ErrorCodes::DealStateNotWithBond.into())
        }
    }
}
