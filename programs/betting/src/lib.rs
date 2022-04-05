use anchor_lang::{prelude::*, solana_program::clock};
use anchor_lang::solana_program::{program::invoke, program::invoke_signed, system_instruction };
pub mod error;
use crate::{error::BetError};
use pyth_client;
use std::mem::size_of;
declare_id!("BBUyeLU1S2gU8fFfnCJrTeEWx76V6risLQGyDjRqr1yn");

const ADMIN_FEE: u8 = 30; // 3%
const TREASURY_ACCOUNT: &str = "HeGTsuhcCpuHnia7tNyq8JZKZ4CTQ71w9WcGf23h9kMG";

#[program]
pub mod betting {
    use super::*;
    pub fn initialize_bet(
        ctx: Context<InitializeBet>,
        bet_id: u64,
        bet_amount: u64,
        betting_period: u32
    ) -> Result<()> {
        if ctx.accounts.bet_detail_account.is_initial {
            return Err(ProgramError::AccountAlreadyInitialized.into());
        }
        if ctx.accounts.bet_detail_account.is_start {
            return Err(error!(BetError::AlreadyStart));
        }
        if ctx.accounts.bet_detail_account.is_finish {
            return Err(error!(BetError::AlreadyEnd));
        }
        if betting_period <= 0 {
            return Err(error!(BetError::WrongBettingPeriod));
        }
        if bet_amount <= 0 {
            return Err(error!(BetError::WrongBetAmount));
        }
        
        ctx.accounts.bet_detail_account.is_initial = true;
        ctx.accounts.bet_detail_account.admin_account = ctx.accounts.admin_account.key();
        ctx.accounts.bet_detail_account.bet_amount_per_user = bet_amount;
        ctx.accounts.bet_detail_account.bet_id = bet_id;
        ctx.accounts.bet_detail_account.is_start = false;
        ctx.accounts.bet_detail_account.is_finish = false;
        ctx.accounts.bet_detail_account.betting_period = betting_period;
        Ok(())
    }

    pub fn user_bet(
        ctx: Context<UserBet>,
        bet_id: u64,
        bet_predict_result: bool, // true: Price UP, false: Price Down
        locked_price: u64
    ) -> Result<()> {
        if !ctx.accounts.bet_detail_account.is_initial {
            return Err(ProgramError::UninitializedAccount.into());
        }
        if ctx.accounts.user_bet_detail_account.is_initial {
            return Err(ProgramError::AccountAlreadyInitialized.into());
        }
        if ctx.accounts.bet_detail_account.is_start {
            return Err(error!(BetError::AlreadyStart));
        }
        if ctx.accounts.bet_detail_account.is_finish {
            return Err(error!(BetError::AlreadyEnd));
        }
        if bet_id != ctx.accounts.bet_detail_account.bet_id {
            return Err(error!(BetError::WrongBetId));
        }
        if **ctx.accounts.user_account.lamports.borrow() < ctx.accounts.bet_detail_account.bet_amount_per_user {
            return Err(error!(BetError::NoEnoughSol));
        }
        
        invoke(
            &system_instruction::transfer(
                ctx.accounts.user_account.key,
                ctx.accounts.escrow_account.key,
                ctx.accounts.bet_detail_account.bet_amount_per_user,
            ),
            &[
                ctx.accounts.user_account.to_account_info().clone(),
                ctx.accounts.escrow_account.clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
        )?;
        

        ctx.accounts.user_bet_detail_account.is_initial = true;
        ctx.accounts.user_bet_detail_account.user_account = ctx.accounts.user_account.key();
        ctx.accounts.user_bet_detail_account.bet_pair = ctx.accounts.bet_pyth_account.key();
        ctx.accounts.user_bet_detail_account.bet_predict = bet_predict_result;
        ctx.accounts.user_bet_detail_account.bet_id = bet_id;
        ctx.accounts.user_bet_detail_account.locked_price = locked_price;
        
        ctx.accounts.bet_detail_account.total_betters += 1;
        Ok(())
    }

    pub fn start_bet(
        ctx: Context<StartBet>,
        bet_id: u64,
    ) -> Result<()> {
        if !ctx.accounts.bet_detail_account.is_initial {
            return Err(ProgramError::UninitializedAccount.into());
        }
        if ctx.accounts.bet_detail_account.is_start {
            return Err(error!(BetError::AlreadyStart));
        }
        if ctx.accounts.bet_detail_account.is_finish {
            return Err(error!(BetError::AlreadyEnd));
        }
        if bet_id != ctx.accounts.bet_detail_account.bet_id {
            return Err(error!(BetError::WrongBetId));
        }

        let clock = clock::Clock::get().unwrap();

        ctx.accounts.bet_detail_account.is_start = true;
        ctx.accounts.bet_detail_account.start_time = clock.unix_timestamp;
        Ok(())
    }

    pub fn complete_bet<'info>(
        ctx: Context<CompleteBet>,
        bet_id: u64,
    ) -> Result<()> {
        if !ctx.accounts.bet_detail_account.is_initial {
            return Err(ProgramError::UninitializedAccount.into());
        }
        if !ctx.accounts.bet_detail_account.is_start {
            return Err(BetError::NoStart.into());
        }
        if ctx.accounts.bet_detail_account.is_finish {
            return Err(BetError::AlreadyEnd.into());
        }
        if bet_id != ctx.accounts.bet_detail_account.bet_id {
            return Err(BetError::WrongBetId.into());
        }
        let clock = clock::Clock::get().unwrap();
        let cur_time = clock.unix_timestamp as u64;
        if cur_time < ctx.accounts.bet_detail_account.start_time as u64 + ctx.accounts.bet_detail_account.betting_period as u64 * 60 as u64 {
            return Err(BetError::NoBetEnd.into());
        }
        ctx.accounts.bet_detail_account.is_finish = true;
        Ok(())
    }

    pub fn deterimine_bet_result<'info>(
        ctx: Context<DeterimineBetResult>,
        bet_id: u64,
    ) -> Result<()> {
        if !ctx.accounts.bet_detail_account.is_initial {
            return Err(ProgramError::UninitializedAccount.into());
        }
        if !ctx.accounts.user_bet_detail_account.is_initial {
            return Err(ProgramError::UninitializedAccount.into());
        }
        
        if !ctx.accounts.bet_detail_account.is_start {
            return Err(error!(BetError::NoStart));
        }
        if !ctx.accounts.bet_detail_account.is_finish {
            return Err(error!(BetError::ProgressBet));
        }
        if bet_id != ctx.accounts.bet_detail_account.bet_id {
            return Err(error!(BetError::WrongBetId));
        }

        let pyth_price_info = &ctx.accounts.bet_pyth_account;
        let pyth_price_data = &pyth_price_info.try_borrow_data()?;
        let pyth_price = pyth_client::cast::<pyth_client::Price>(pyth_price_data);

        let mut bet_result = false; // true: Win, false: lose

        if ( ctx.accounts.user_bet_detail_account.locked_price <= pyth_price.agg.price as u64
            && ctx.accounts.user_bet_detail_account.bet_predict ) ||  
            ( ctx.accounts.user_bet_detail_account.locked_price >= pyth_price.agg.price as u64
            && !ctx.accounts.user_bet_detail_account.bet_predict ) {
                bet_result = true;
            }
        
        if bet_result {
            ctx.accounts.bet_detail_account.total_winners += 1;
            ctx.accounts.user_bet_detail_account.is_win = true;
        } else {
            ctx.accounts.bet_detail_account.total_losers += 1;
            ctx.accounts.user_bet_detail_account.is_win = false;
        }

        Ok(())
    }

    pub fn distribute_prize (
        ctx: Context<DistributePrize>,
        bet_id: u64,
        escrow_nonce: u8,
    ) -> Result<()> {
        if ctx.accounts.user_bet_detail_account.is_win {

            if !ctx.accounts.bet_detail_account.is_initial {
                return Err(ProgramError::UninitializedAccount.into());
            }
            if !ctx.accounts.user_bet_detail_account.is_initial {
                return Err(ProgramError::UninitializedAccount.into());
            }
            
            if !ctx.accounts.bet_detail_account.is_start {
                return Err(error!(BetError::NoStart));
            }
            if !ctx.accounts.bet_detail_account.is_finish {
                return Err(error!(BetError::ProgressBet));
            }
            if bet_id != ctx.accounts.bet_detail_account.bet_id {
                return Err(error!(BetError::WrongBetId));
            }
            if ctx.accounts.bet_detail_account.total_betters != ctx.accounts.bet_detail_account.total_winners + ctx.accounts.bet_detail_account.total_losers {
                return Err(error!(BetError::TotalBettersWrong));
            }

            let win_total_bet_amount = ctx.accounts.bet_detail_account.total_losers as u64 * ctx.accounts.bet_detail_account.bet_amount_per_user;

            let market_fee = win_total_bet_amount * ADMIN_FEE as u64 / 1000 as u64;

            if ctx.accounts.bet_detail_account.total_winners == 0 {
                return Err(error!(BetError::NoWinners));
            }

            let win_prize_amount = win_total_bet_amount.checked_sub(market_fee).unwrap().checked_div(ctx.accounts.bet_detail_account.total_winners as u64).unwrap().checked_add(ctx.accounts.bet_detail_account.bet_amount_per_user).unwrap();

            if **ctx.accounts.escrow_account.lamports.borrow() < win_prize_amount {
                return Err(BetError::WrongTotalBetAmount.into());
            }

            invoke_signed(
                &system_instruction::transfer(
                    ctx.accounts.escrow_account.key,
                    ctx.accounts.user_account.key,
                    win_prize_amount,
                ),
                &[
                    ctx.accounts.escrow_account.clone(),
                    ctx.accounts.user_account.to_account_info().clone(),
                    ctx.accounts.system_program.to_account_info().clone(),
                ],
                &[&[b"hypothese-escrow", &bet_id.to_le_bytes(), &[escrow_nonce]]],
            )?;

        }
        
        Ok(())
    }

    pub fn reward_admin (
        ctx: Context<RewardAdmin>,
        bet_id: u64,
        escrow_nonce: u8
    ) -> Result<()> {
        if !ctx.accounts.bet_detail_account.is_initial {
            return Err(ProgramError::UninitializedAccount.into());
        }
        if !ctx.accounts.bet_detail_account.is_start {
            return Err(error!(BetError::NoStart));
        }
        if !ctx.accounts.bet_detail_account.is_finish {
            return Err(error!(BetError::ProgressBet));
        }
        if bet_id != ctx.accounts.bet_detail_account.bet_id {
            return Err(error!(BetError::WrongBetId));
        }

        let win_total_bet_amount = ctx.accounts.bet_detail_account.total_losers as u64 * ctx.accounts.bet_detail_account.bet_amount_per_user;

        let market_fee = win_total_bet_amount * ADMIN_FEE as u64 / 1000 as u64;

        if **ctx.accounts.escrow_account.lamports.borrow() < market_fee {
            return Err(BetError::WrongTotalBetAmount.into());
        }

        invoke_signed(
            &system_instruction::transfer(
                ctx.accounts.escrow_account.key,
                ctx.accounts.treasury_account.key,
                market_fee,
            ),
            &[
                ctx.accounts.escrow_account.clone(),
                ctx.accounts.treasury_account.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            ],
            &[&[b"hypothese-escrow", &bet_id.to_le_bytes(), &[escrow_nonce]]],
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(bet_id: u64)]
pub struct InitializeBet<'info> {
    #[account(mut)]
    pub admin_account: Signer<'info>,
    #[account(
        init,
        seeds = [
            b"bet-detail".as_ref(),
            &bet_id.to_le_bytes(),
            admin_account.key().as_ref()
        ],
        bump,
        payer = admin_account,
        space = 8 + size_of::<BetDetails>(),
    )]
    pub bet_detail_account: Box<Account<'info, BetDetails>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(bet_id: u64)]
pub struct UserBet<'info> {
    #[account(mut)]
    pub user_account: Signer<'info>,
    #[account(
        mut,
        constraint = bet_id == bet_detail_account.bet_id
    )]
    pub bet_detail_account: Box<Account<'info, BetDetails>>,
    #[account(
        mut,
        seeds = [
            b"hypothese-escrow".as_ref(),
            &bet_id.to_le_bytes(),
        ],
        bump,
    )]
    pub escrow_account: AccountInfo<'info>,
    #[account(
        init,
        seeds = [
            b"user-bet".as_ref(),
            &bet_id.to_le_bytes(),
            bet_pyth_account.key().as_ref(),
            user_account.key().as_ref()
        ],
        bump,
        payer = user_account,
        space = 8 + size_of::<UserBetDetails>(),
    )]
    pub user_bet_detail_account: Box<Account<'info, UserBetDetails>>,
    pub bet_pyth_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(bet_id: u64)]
pub struct StartBet<'info> {
    #[account(mut)]
    pub admin_account: Signer<'info>,
    #[account(
        mut,
        has_one = admin_account
    )]
    pub bet_detail_account: Box<Account<'info, BetDetails>>,
}

#[derive(Accounts)]
#[instruction(bet_id: u64)]
pub struct CompleteBet<'info> {
    #[account(mut)]
    pub admin_account: Signer<'info>,
    #[account(
        mut,
        has_one = admin_account
    )]
    pub bet_detail_account: Box<Account<'info, BetDetails>>,
}

#[derive(Accounts)]
#[instruction(bet_id: u64)]
pub struct DeterimineBetResult<'info> {
    #[account(mut)]
    pub user_account: Signer<'info>,
    #[account(
        mut,
        constraint = bet_id == bet_detail_account.bet_id
    )]
    pub bet_detail_account: Box<Account<'info, BetDetails>>,
    
    #[account(
        mut,
        constraint = user_bet_detail_account.user_account == user_account.key(),
        constraint = user_bet_detail_account.bet_pair == bet_pyth_account.key(),
        constraint = user_bet_detail_account.bet_id == bet_id,
    )]
    pub user_bet_detail_account: Box<Account<'info, UserBetDetails>>,
    pub bet_pyth_account: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(bet_id: u64)]
pub struct DistributePrize<'info> {
    #[account(mut)]
    pub user_account: Signer<'info>,
    #[account(
        mut,
        constraint = bet_id == bet_detail_account.bet_id
    )]
    pub bet_detail_account: Box<Account<'info, BetDetails>>,
    #[account(
        mut,
        seeds = [
            b"hypothese-escrow".as_ref(),
            &bet_id.to_le_bytes(),
        ],
        bump
    )]
    pub escrow_account: AccountInfo<'info>,
    #[account(
        mut,
        constraint = user_bet_detail_account.user_account == user_account.key(),
        constraint = user_bet_detail_account.bet_id == bet_id,
        constraint = user_bet_detail_account.bet_pair == bet_pyth_account.key(),
        close = user_account
    )]
    pub user_bet_detail_account: Box<Account<'info, UserBetDetails>>,
    pub bet_pyth_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(bet_id: u64)]
pub struct RewardAdmin<'info> {
    #[account(mut)]
    pub admin_account: Signer<'info>,
    #[account(
        mut,
        constraint = bet_id == bet_detail_account.bet_id,
        has_one = admin_account
    )]
    pub bet_detail_account: Box<Account<'info, BetDetails>>,
    #[account(
        mut,
        seeds = [
            b"hypothese-escrow".as_ref(),
            &bet_id.to_le_bytes(),
        ],
        bump
    )]
    pub escrow_account: AccountInfo<'info>,
    #[account(
        mut,
        constraint = treasury_account.key() == TREASURY_ACCOUNT.parse::<Pubkey>().unwrap()
    )]
    pub treasury_account: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}


#[account]
#[repr(C)]
pub struct BetDetails {
    pub is_initial : bool,
    pub admin_account: Pubkey,
    pub start_time: i64,
    pub bet_amount_per_user: u64,
    pub bet_id: u64,
    pub is_start: bool,
    pub is_finish: bool,
    pub betting_period: u32,
    pub total_betters: u16,
    pub total_winners: u16,
    pub total_losers: u16,
}

#[repr(C)]
#[account]
pub struct UserBetDetails {
    pub is_initial : bool,
    pub user_account: Pubkey,
    pub bet_pair: Pubkey,
    pub bet_predict: bool, // true: UP, false: DOWN
    pub bet_id: u64,
    pub locked_price: u64,
    pub is_win : bool,
    pub get_prize: bool
}