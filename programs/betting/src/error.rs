use anchor_lang::prelude::*;

#[error_code]
pub enum BetError {
    #[msg("The setted price is wrong")]
    WrongPrice,
    #[msg("The bet amount is wrong")]
    WrongBetAmount,
    #[msg("The bet round is already started")]
    AlreadyStart,
    #[msg("The bet round is already finished")]
    AlreadyEnd,
    #[msg("The betting period is wrong")]
    WrongBettingPeriod,
    #[msg("User doesn't have enough SOL")]
    NoEnoughSol,
    #[msg("Betting id isn't wrong")]
    WrongBetId,
    #[msg("Total Betting amount is wrong")]
    WrongTotalBetAmount,
    #[msg("The betting user ratio should be 6:4")]
    WrongRatio,
    #[msg("The betting round is not started yet")]
    NoStart,
    #[msg("Betting is not ended yet")]
    NoBetEnd,
    #[msg("Check betting result again")]
    WrongBetResult,
    #[msg("The betting round is not finished yet.")]
    ProgressBet,
    #[msg("Total Winners and Losers are wrong")]
    TotalBettersWrong,
    #[msg("There are no winners")]
    NoWinners,
}