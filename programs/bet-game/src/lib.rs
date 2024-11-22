use anchor_lang::error_code;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak::hash;
use std::mem::size_of;
// clock
use anchor_lang::solana_program::clock::Clock;
declare_id!("A1t82ktKi5mXWWau4zBCN1LxP6xiWQ8VRo87N34YvFdD");

pub const GLOBAL_STATE_SEED: &[u8] = b"GLOBAL-STATE-SEED";
pub const ROUND_STATE_SEED: &[u8] = b"ROUND-STATE-SEED";
pub const MAX_REVEAL_TIME: i64 = 10 * 60; // 10 minutes

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
        hashed_num: [u8; 32],
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.total_round += 1;
        let round_state = &mut ctx.accounts.round_state;
        round_state.round_index = round_index;
        round_state.creator = *ctx.accounts.user.key;
        round_state.status = false;
        round_state.start_time = Clock::get()?.unix_timestamp;
        round_state.creator_hash = hashed_num;
        Ok(())
    }

    /// Join the round with a number
    pub fn join_round(ctx: Context<Join>, round_index: u32, num: u32) -> Result<()> {
        require!(
            *ctx.accounts.user.key != ctx.accounts.round_state.creator,
            BetGame::NotCreator
        );
        require!(
            ctx.accounts.round_state.joiner == Pubkey::default(),
            BetGame::AlreadyJoined
        );
        let round_state = &mut ctx.accounts.round_state;
        round_state.joiner = *ctx.accounts.user.key;
        round_state.joiner_num = num;
        round_state.join_time = Clock::get()?.unix_timestamp;
        Ok(())
    }
    /// Reveal the number and determine the winner
    pub fn reveal(ctx: Context<Reveal>, round_index: u32, num: u32) -> Result<()> {
        msg!("Creator {:?}", ctx.accounts.round_state.creator);

        require!(
            *ctx.accounts.user.key == ctx.accounts.round_state.creator,
            BetGame::NotCreator
        );
        msg!("hashed_num {:?}", hash(&num.to_le_bytes()).to_bytes());
        require!(
            hash(&num.to_le_bytes()).to_bytes() == ctx.accounts.round_state.creator_hash,
            BetGame::HashNotMatch
        );
        require!(
            !ctx.accounts.round_state.is_revealed,
            BetGame::AlreadyRevealed
        );
        require!(
            Clock::get()?.unix_timestamp - ctx.accounts.round_state.join_time < MAX_REVEAL_TIME,
            BetGame::OutOfTime
        );
        require!(
            ctx.accounts.round_state.joiner != Pubkey::default(),
            BetGame::NoJoiner
        );

        let round_state = &mut ctx.accounts.round_state;
        round_state.creator_num = num;
        round_state.is_revealed = true;
        let winner = if round_state.creator_num > round_state.joiner_num {
            round_state.creator
        } else {
            round_state.joiner
        };
        round_state.winner = winner;
        Ok(())
    }
    // In case the creator doesn't reveal the number, the joiner can claim the prize
    pub fn claim(ctx: Context<Claim>, round_index: u32) -> Result<()> {
        let round_state = &mut ctx.accounts.round_state;
        require!(round_state.is_revealed, BetGame::AlreadyRevealed);
        require!(
            Clock::get()?.unix_timestamp - round_state.join_time > MAX_REVEAL_TIME,
            BetGame::NotEndRevealTime
        );
        let winner = round_state.joiner;
        round_state.winner = winner;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = user,  seeds = [GLOBAL_STATE_SEED], bump, space = 8 + size_of::<GlobalState>())]
    pub global_state: Account<'info, GlobalState>,
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK:
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(round_index: u32, hashed_num: [u8; 32])]
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
        init,
        seeds = [ROUND_STATE_SEED, &round_index.to_le_bytes()],
        bump,
        payer = user,
        space = 8 + size_of::<RoundState>()
    )]
    pub round_state: Account<'info, RoundState>,
    /// CHECK:
    pub system_program: AccountInfo<'info>,
}

#[derive(Accounts)]
#[instruction(round_index: u32, num: u32)]
pub struct Join<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [ROUND_STATE_SEED, &round_index.to_le_bytes()],
        bump
    )]
    pub round_state: Account<'info, RoundState>,
}

#[derive(Accounts)]
#[instruction(round_index: u32, num: u32)]
pub struct Reveal<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        seeds = [ROUND_STATE_SEED, &round_index.to_le_bytes()],
        bump
    )]
    pub round_state: Account<'info, RoundState>,
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
pub struct RoundState {
    pub round_index: u32,
    pub creator: Pubkey,
    pub joiner: Pubkey,
    pub status: bool,
    pub creator_hash: [u8; 32],
    pub creator_num: u32,
    pub joiner_num: u32,
    pub start_time: i64,
    pub join_time: i64,
    pub is_revealed: bool,
    pub winner: Pubkey,
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
}
