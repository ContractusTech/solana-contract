use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCodes {
    #[msg("The fee is too low")]
    FeeIsTooLow,
    #[msg("Holder mode unavailable")]
    HolderModeUnavailable,
    #[msg("The amount is too small.")]
    AmountTooLow,
    #[msg("The deadline has not yet come.")]
    DeadlineNotExpired,
    #[msg("Deadline expired")]
    DeadlineExpired,
    #[msg("Not implemented method")]
    NotImplemented,

    #[msg("DealStateNotWithChecker")]
    DealStateNotWithChecker,
    #[msg("DealStateWithChecker")]
    DealStateWithChecker,

    #[msg("NoClientBond")]
    NoClientBond,
    #[msg("NoExecutorBond")]
    NoExecutorBond,

    #[msg("InvalidMint")]
    InvalidMint,
    #[msg("InvalidOwner")]
    InvalidOwner,

    #[msg("DealWithClientBond")]
    DealWithClientBond,

    #[msg("DealWithExecutorBond")]
    DealWithExecutorBond,
}

#[error_code]
pub enum InvalidAccount {
    #[msg("Initializer")]
    Initializer,
    #[msg("Checker")]
    Checker,
    #[msg("CheckerDealTokenAccount")]
    CheckerDealTokenAccount,

    #[msg("ClientBondMint")]
    ClientBondMint,
    #[msg("ExecutorBondMint")]
    ExecutorBondMint,
    #[msg("DealStateClientBondMint")]
    DealStateClientBondMint,
    #[msg("DealStateExecutorBondMint")]
    DealStateExecutorBondMint,
    #[msg("DealStateHolderTokenAccount")]
    DealStateHolderTokenAccount,

    #[msg("ClientBondTokenAccount")]
    ClientBondTokenAccount,
    #[msg("ExecutorBondTokenAccount")]
    ExecutorBondTokenAccount,

    #[msg("DealStateClientBondTokenAccount")]
    DealStateClientBondTokenAccount,
    #[msg("DealStateExecutorBondTokenAccount")]
    DealStateExecutorBondTokenAccount,

    #[msg("ClientHolderTokenAccount")]
    ClientHolderTokenAccount,
    #[msg("ClientHolderTokenAccountMint")]
    ClientHolderTokenAccountMint,

    #[msg("ClientHolderTokenAccountOwner")]
    ClientHolderTokenAccountOwner,
}
