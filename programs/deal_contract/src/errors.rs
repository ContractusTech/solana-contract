use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCodes {
    #[msg("The deal already started")]
    AlreadyStarted,
    #[msg("Th deal not started")]
    NotStarted,
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
    #[msg("The deal need cancel with bond")]
    NeedCancelWithBond,
    #[msg("The deal need cancel without bond")]
    NeedCancelWithoutBond,
    #[msg("Not implemented method")]
    NotImplemented,

    #[msg("DealStateNotWithChecker")]
    DealStateNotWithChecker,

    #[msg("NoClientBond")]
    NoClientBond,
    #[msg("NoExecutorBond")]
    NoExecutorBond,
}

#[error_code]
pub enum InvalidAccount {
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

    #[msg("ClientHolderTokenAccountMint")]
    ClientHolderTokenAccountMint,

    #[msg("ClientHolderTokenAccountOwner")]
    ClientHolderTokenAccountOwner,
}

#[error_code]
pub enum TransferError {
    #[msg("ServiceFee")]
    ServiceFee,
}

// #[error_code]
// pub enum TraitError {
//     #[msg("DealState")]
//     DealState,

//     #[msg("DealStateDealTokenAccount")]
//     DealStateDealTokenAccount,

//     #[msg("DealStateBondTokenAccount")]
//     DealStateBondTokenAccount,

//     #[msg("Client")]
//     Client,

//     #[msg("ClientDealTokenAccount")]
//     ClientDealTokenAccount,

//     #[msg("ClientDealTokenAccount")]
//     ClientHolderTokenAccount,

//     #[msg("ClientBondTokenAccount")]
//     ClientBondTokenAccount,

//     #[msg("ClientServiceTokenAccount")]
//     ClientServiceTokenAccount,

//     #[msg("Executor")]
//     Executor,

//     #[msg("ExecutorDealTokenAccount")]
//     ExecutorDealTokenAccount,

//     #[msg("ExecutorBondTokenAccount")]
//     ExecutorBondTokenAccount,

//     #[msg("Checker")]
//     Checker,

//     #[msg("CheckerBondTokenAccount")]
//     CheckerBondTokenAccount,

//     #[msg("CheckerDealTokenAccount")]
//     CheckerDealTokenAccount,

//     #[msg("TokenProgram")]
//     TokenProgram,

//     #[msg("DealMint")]
//     DealMint,

//     #[msg("ServiceMint")]
//     ServiceMint,

//     #[msg("HolderMint")]
//     HolderMint,

//     #[msg("ClientBondMint")]
//     ClientBondMint,

//     #[msg("ExecutorBondMint")]
//     ExecutorBondMint,
// }
