use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::*;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(mut, constraint = pool.underlying == user_account.mint)]
    pub pool: Account<'info, Pool>,
    #[account(mut, constraint = depositor.key() == user_info.authority)]
    pub user_info: Account<'info, Depositor>,
    #[account(mut, constraint = pool_account.key() == pool.pool_account)]
    pub pool_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(seeds = [b"state"], bump = state.bump)]
    pub state: Account<'info, Genesis>,
    #[account(mut, constraint = pool.underlying == user_account.mint)]
    pub pool: Account<'info, Pool>,
    #[account(mut, seeds = [b"vault"], bump = state.vault_bump)]
    pub vault: SystemAccount<'info>,
    #[account(mut, constraint = depositor.key() == user_info.authority)]
    pub user_info: Account<'info, Depositor>,
    #[account(mut, constraint = pool_account.key() == pool.pool_account)]
    pub pool_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(seeds = [b"state"], bump = state.bump)]
    pub state: Account<'info, Genesis>,
    #[account(seeds = [b"vault"], bump = state.vault_bump)]
    pub vault: SystemAccount<'info>,
    #[account(constraint = pool_underlying.key() == pool.underlying)]
    pub pool_underlying: Account<'info, Mint>,
    #[account(mut, seeds = [pool_underlying.key().as_ref()], bump = pool.bump)]
    pub pool: Account<'info, Pool>,
    #[account(mut, constraint = depositor.key() == user_info.authority)]
    pub user_info: Account<'info, Depositor>,
    #[account(mut, constraint = pool_account.key() == pool.pool_account)]
    pub pool_account: Account<'info, TokenAccount>,
    #[account(mut, constraint = reward_mint.key() == state.reward_mint)]
    pub reward_mint: Account<'info, Mint>,
    #[account(mut, constraint = user_reward_account.mint == state.reward_mint)]
    pub user_reward_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub clock: Sysvar<'info, Clock>
}

#[derive(Accounts)]
pub struct AddPool<'info> {
    #[account(mut, constraint = authority.key() == state.authority)]
    pub authority: Signer<'info>,
    #[account(seeds = [b"state"], bump = state.bump)]
    pub state: Account<'info, Genesis>,
    #[account(seeds = [b"vault"], bump = state.vault_bump)]
    pub vault: SystemAccount<'info>,
    #[account(init, space = 8 + Pool::POOL_SIZE, payer = authority, seeds = [token_mint.key().as_ref()], bump)]
    pub pool: Account<'info, Pool>,
    #[account(init, payer = authority, token::mint = token_mint, token::authority = vault)]
    pub pool_account: Account<'info, TokenAccount>,
    pub token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub clock: Sysvar<'info, Clock>
}

#[derive(Accounts)]
pub struct InitializeGenesis<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(seeds = [b"vault"], bump)]
    pub vault: SystemAccount<'info>,
    #[account(init, space = 8 + Genesis::STATE_SIZE, payer = authority, seeds = [b"state"], bump)]
    pub state: Account<'info, Genesis>,
    #[account(init, payer = authority, token::mint = reward_mint, token::authority = vault, seeds = [b"rewards"], bump)]
    pub reward_account: Account<'info, TokenAccount>,
    pub reward_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}
