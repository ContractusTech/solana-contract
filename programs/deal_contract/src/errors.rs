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
    DeadlineNotCome,
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

    #[msg("DealStateNotWithBond")]
    DealStateNotWithBond,
}
