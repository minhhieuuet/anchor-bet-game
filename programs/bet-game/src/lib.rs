use anchor_lang::error_code;
use anchor_lang::prelude::*;
use std::mem::size_of;
use anchor_lang::solana_program::{program::invoke, system_instruction};

// clock
use anchor_lang::solana_program::clock::Clock;
declare_id!("Ed477P75RHCSFhnWkY2TWGTXuwXpUfnG5BjQ3QzLnntH");

pub const GLOBAL_STATE_SEED: &[u8] = b"GLOBAL-STATE-SEED";
pub const ROUND_STATE_SEED: &[u8] = b"ROUND-STATE-SEED";
pub const VAULT_SEED: &[u8] = b"VAULT_SEED";

pub const ROUND_DURATION: i64 = 24 * 60 * 60; // 24 hours
pub const FEE: u64 = 10000000; // 0.01 SOL

#[program]
pub mod bet_game {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    /// Create a new round
    pub fn create_round(
        ctx: Context<Create>,
        round_index: u32,
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        require!(
            round_index == global_state.total_round,
            BetGame::InvalidRoundIndex
        );
        global_state.total_round += 1;
        let round_state = &mut ctx.accounts.round_state;
        round_state.round_index = round_index;
        round_state.creator = *ctx.accounts.user.key;
        round_state.start_time = Clock::get()?.unix_timestamp;
        round_state.timeout = Clock::get()?.unix_timestamp + ROUND_DURATION;
         // Transfer fee to vault
         let _ = invoke(
            &system_instruction::transfer(
                ctx.accounts.user.key,
                ctx.accounts.vault.key,
                FEE,
            ),
            &[
                ctx.accounts.user.to_account_info().clone(),
                ctx.accounts.vault.clone(),
                ctx.accounts.system_program.clone(),
            ],
        );
        Ok(())
    }

    pub fn creator_update_score(ctx: Context<CreatorUpdateScore>, round_index: u32, score: u32) -> Result<()> {
        let round_state = &mut ctx.accounts.round_state;
        require!(round_state.creator == *ctx.accounts.user.key, BetGame::NotCreator);
        require!(round_state.is_finished == false, BetGame::AlreadyFinished);
        require!(Clock::get()?.unix_timestamp < round_state.timeout, BetGame::OutOfTime);
        // require!(round_state.joiner != Pubkey::default(), BetGame::NoJoiner);
        round_state.creator_score = score;
        round_state.is_creator_updated = true;
        if round_state.is_joiner_updated == true {
            round_state.is_finished = true;
            if round_state.creator_score > round_state.joiner_score {
                round_state.winner = round_state.creator;
            } else if round_state.creator_score < round_state.joiner_score {
                round_state.winner = round_state.joiner;
            }
        }
        Ok(())
    }

    /// Join the round with a number
    pub fn join_round(ctx: Context<Join>, round_index: u32) -> Result<()> {
        require!(
            ctx.accounts.round_state.joiner == Pubkey::default(),
            BetGame::AlreadyJoined
        );
        require!(
            ctx.accounts.round_state.is_finished == false,
            BetGame::AlreadyFinished
        );
        let round_state = &mut ctx.accounts.round_state;
        round_state.joiner = *ctx.accounts.user.key;
        round_state.join_time = Clock::get()?.unix_timestamp;
        // Transfer fee to vault
        let _ = invoke(
            &system_instruction::transfer(
                ctx.accounts.user.key,
                ctx.accounts.vault.key,
                FEE,
            ),
            &[
                ctx.accounts.user.to_account_info().clone(),
                ctx.accounts.vault.clone(),
                ctx.accounts.system_program.clone(),
            ],
        );
        Ok(())
    }

    pub fn joiner_update_score(ctx: Context<JoinerUpdateScore>, round_index: u32, score: u32) -> Result<()> {
        let round_state = &mut ctx.accounts.round_state;
        require!(round_state.joiner == *ctx.accounts.user.key, BetGame::NotJoiner);
        require!(round_state.is_finished == false, BetGame::AlreadyFinished);
        require!(Clock::get()?.unix_timestamp < round_state.timeout, BetGame::OutOfTime);
        round_state.joiner_score = score;
        round_state.is_joiner_updated = true;
        round_state.join_time = Clock::get()?.unix_timestamp;
        if round_state.is_creator_updated == true {
            round_state.is_finished = true;
            if round_state.creator_score > round_state.joiner_score {
                round_state.winner = round_state.creator;
            } else if round_state.creator_score < round_state.joiner_score {
                round_state.winner = round_state.joiner;
            }
        }
        Ok(())
    }

    pub fn claim(ctx: Context<Claim>, round_index: u32) -> Result<()> {
        let round_state = &mut ctx.accounts.round_state;
        require!(round_state.creator == *ctx.accounts.creator.key, BetGame::WrongCreator);
        require!(round_state.joiner == *ctx.accounts.joiner.key, BetGame::WrongJoiner);
        require!(round_state.is_finished == true, BetGame::NotEndYet);
        require!(round_state.is_creator_updated == true, BetGame::CreatorNotUpdated);
        require!(round_state.is_joiner_updated == true, BetGame::CreatorNotUpdated);
        require!(round_state.is_claimed == false, BetGame::AlreadyClaimed);
        // Transfer the deposit to the winner
        if round_state.winner == round_state.creator {
            **ctx.accounts.vault.lamports.borrow_mut() -= FEE * 2;
            **ctx.accounts.creator.lamports.borrow_mut() += FEE * 2;
        } else {
            **ctx.accounts.vault.lamports.borrow_mut() -= FEE * 2;
            **ctx.accounts.joiner.lamports.borrow_mut() += FEE * 2;
        }
        // If draw, transfer the deposit back
        if round_state.creator_score == round_state.joiner_score {
            **ctx.accounts.vault.lamports.borrow_mut() -= FEE * 2;
            **ctx.accounts.creator.lamports.borrow_mut() += FEE;
            **ctx.accounts.joiner.lamports.borrow_mut() += FEE;
        }
        round_state.is_claimed = true;
        Ok(())
    }

    // Incase timeout, no one join the round, the creator can claim the deposit
    pub fn claim_deposit(ctx: Context<ClaimDeposit>, round_index: u32) -> Result<()> {
        let round_state = &mut ctx.accounts.round_state;
        require!(round_state.creator == *ctx.accounts.user.key, BetGame::NotCreator);
       // require timeout
        require!(Clock::get()?.unix_timestamp > round_state.timeout, BetGame::NotOutOfTime);
        require!(
            round_state.joiner == Pubkey::default(),
            BetGame::NoJoiner
        );
        // Transfer the deposit back to the creator
        **ctx.accounts.vault.lamports.borrow_mut() -= FEE;
        **ctx.accounts.user.lamports.borrow_mut() += FEE;
        round_state.is_finished = true;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user,  seeds = [GLOBAL_STATE_SEED], bump, space = 8 + size_of::<GlobalState>())]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        init,
        seeds = [VAULT_SEED],
        bump,
        payer = user,
        space = 8 + size_of::<GlobalState>()
    )]
    /// CHECK: this should be checked with vault address
    pub vault: AccountInfo<'info>,
    /// CHECK:
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(round_index: u32)]
pub struct Create<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [GLOBAL_STATE_SEED],
        bump,
    )]
    pub global_state: Account<'info, GlobalState>,
    #[account(
        init_if_needed,
        seeds = [ROUND_STATE_SEED, &round_index.to_le_bytes()],
        bump,
        payer = user,
        space = 8 + size_of::<RoundState>()
    )]
    pub round_state: Account<'info, RoundState>,
    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump,
    )]
    /// CHECK: this should be checked with vault address
    pub vault: AccountInfo<'info>,
    /// CHECK:
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(round_index: u32)]
pub struct Join<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [ROUND_STATE_SEED, &round_index.to_le_bytes()],
        bump
    )]
    pub round_state: Account<'info, RoundState>,
    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump,
    )]
    /// CHECK: this should be checked with vault address
    pub vault: AccountInfo<'info>,
    /// CHECK:
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(round_index: u32)]
pub struct CreatorUpdateScore<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [ROUND_STATE_SEED, &round_index.to_le_bytes()],
        bump
    )]
    pub round_state: Account<'info, RoundState>,
    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump,
    )]
    /// CHECK: this should be checked with vault address
    pub vault: AccountInfo<'info>,
    /// CHECK:
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(round_index: u32)]
pub struct JoinerUpdateScore<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [ROUND_STATE_SEED, &round_index.to_le_bytes()],
        bump
    )]
    pub round_state: Account<'info, RoundState>,
    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump,
    )]
    /// CHECK: this should be checked with vault address
    pub vault: AccountInfo<'info>,
    /// CHECK:
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(round_index: u32)]
pub struct Claim<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [ROUND_STATE_SEED, &round_index.to_le_bytes()],
        bump
    )]
    pub round_state: Account<'info, RoundState>,
    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump,
    )]
    /// CHECK: this should be checked with vault address
    pub vault: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    creator: AccountInfo<'info>,
    #[account(mut)]
    /// CHECK:
    joiner: AccountInfo<'info>,
    /// CHECK:
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(round_index: u32)]
pub struct ClaimDeposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [ROUND_STATE_SEED, &round_index.to_le_bytes()],
        bump
    )]
    pub round_state: Account<'info, RoundState>,
    #[account(
        mut,
        seeds = [VAULT_SEED],
        bump,
    )]
    /// CHECK: this should be checked with vault address
    pub vault: AccountInfo<'info>,
    /// CHECK:
    pub system_program: AccountInfo<'info>,
}


#[account]
#[derive(Default)]
pub struct GlobalState {
    pub total_round: u32,
    // Vector of round index
    pub round_index: Vec<u32>,
}

#[account]
#[derive(Default)]
pub struct UserRoundList {
    pub round_indexs: Vec<u32>
}


#[account]
#[derive(Default)]
pub struct RoundState {
    pub round_index: u32,
    pub creator: Pubkey,
    pub joiner: Pubkey,
    pub is_creator_updated: bool,
    pub is_joiner_updated: bool,
    pub creator_score: u32,
    pub joiner_score: u32,
    pub start_time: i64,
    pub join_time: i64,
    pub winner: Pubkey,
    pub timeout: i64,
    pub is_finished: bool,
    pub is_claimed: bool,
}

#[error_code]
pub enum BetGame {
    #[msg("Hash not match")]
    HashNotMatch,
    #[msg("Already revealed")]
    AlreadyRevealed,
    #[msg("Out of time")]
    OutOfTime,
    #[msg("No joiner")]
    NoJoiner,
    #[msg("Not creator")]
    NotCreator,
    #[msg("Not end reveal time yet")]
    NotEndRevealTime,
    #[msg("Already joined")]
    AlreadyJoined,
    #[msg("Not joiner")]
    NotJoiner,
    #[msg("Not end yet")]
    NotEndYet,
    #[msg("Already finished")]
    AlreadyFinished,
    #[msg("Invalid round index")]
    InvalidRoundIndex,
    #[msg("Not out of time")]
    NotOutOfTime,
    #[msg("Creator not updated")]
    CreatorNotUpdated,
    #[msg("Wrong joiner")]
    WrongJoiner,
    #[msg("Wrong creator")]
    WrongCreator,
    #[msg("Already claimed")]
    AlreadyClaimed,
}
