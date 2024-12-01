use anchor_lang::error_code;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::keccak::hash;
use std::mem::size_of;
use anchor_lang::solana_program::{program::invoke, system_instruction};

// clock
use anchor_lang::solana_program::clock::Clock;
declare_id!("V8g165mAF8vQVBzFp7eXYcakwohWMSePrXPDmb6eF4k");

pub const GLOBAL_STATE_SEED: &[u8] = b"GLOBAL-STATE-SEED";
pub const ROUND_STATE_SEED: &[u8] = b"ROUND-STATE-SEED";
pub const VAULT_SEED: &[u8] = b"VAULT_SEED";

pub const MAX_REVEAL_TIME: i64 = 60 * 60; // 60 minutes
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
        // Transfer the prize to the winner
        let _ = invoke(
            &system_instruction::transfer(
                ctx.accounts.vault.key,
                &winner,
                FEE * 2,
            ),
            &[
                ctx.accounts.vault.clone(),
                ctx.accounts.user.to_account_info().clone(),
                ctx.accounts.system_program.clone(),
            ],
        );
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
        // Transfer the prize to the winner
        let _ = invoke(
            &system_instruction::transfer(
                ctx.accounts.vault.key,
                &winner,
                FEE * 2,
            ),
            &[
                ctx.accounts.vault.clone(),
                ctx.accounts.user.to_account_info().clone(),
                ctx.accounts.system_program.clone(),
            ],
        );

        Ok(())
    }

    // Incase timeout, no one join the round, the creator can claim the deposit
    pub fn claim_deposit(ctx: Context<ClaimDeposit>, round_index: u32) -> Result<()> {
        let round_state = &mut ctx.accounts.round_state;
        require!(round_state.creator == *ctx.accounts.user.key, BetGame::NotCreator);
        require!(
            Clock::get()?.unix_timestamp > round_state.timeout,
            BetGame::OutOfTime
        );
        require!(
            round_state.joiner == Pubkey::default(),
            BetGame::NoJoiner
        );
        let creator = round_state.creator;
        // Transfer the deposit back to the creator
        let _ = invoke(
            &system_instruction::transfer(
                ctx.accounts.vault.key,
                &creator,
                FEE,
            ),
            &[
                ctx.accounts.vault.clone(),
                ctx.accounts.user.to_account_info().clone(),
                ctx.accounts.system_program.clone(),
            ],
        );
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
    pub timeout: i64,
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
