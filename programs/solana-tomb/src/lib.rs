use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token::{self, Transfer as SplTransfer, MintTo};

declare_id!("EbUS5R37dPQRdDKgh7Kju9JhwUg6FPB1fSRAjbUj29EQ");

mod state;

pub mod pool;
pub mod errors;
pub mod instructions;
pub mod events;

pub use state::*;
pub use instructions::*;
pub use events::*;

#[program]
pub mod solana_tomb {
    use super::*;

    /// Initializes the genesis pool with all of its accounts.
    pub fn initialize(ctx: Context<InitializeGenesis>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.authority = ctx.accounts.authority.key();
        state.dev_share = ctx.accounts.authority.key(); // TODO: We prob need to create a new token account for this
        state.reward_mint = ctx.accounts.reward_mint.key();

        // Create new vault account.
        let rent_xfer_context = CpiContext::new(
            ctx.accounts.system_program.to_account_info(), 
            system_program::Transfer{
                from: ctx.accounts.authority.to_account_info(),
                to: ctx.accounts.vault.to_account_info()
            }
        );
        let rent_xfer_res = system_program::transfer(rent_xfer_context, 20000000);
        if !rent_xfer_res.is_ok() {
            return err!(errors::GenesisError::SolTransferFailed);
        }

        state.vault = ctx.accounts.vault.key();
        state.bump = ctx.bumps.state;
        state.vault_bump = ctx.bumps.vault;
        Ok(())
    }
    
    /// Adds a new pool to the genesis pool.
    pub fn add_pool(ctx: Context<AddPool>, reward_rate: u64, end_at: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        pool.underlying = ctx.accounts.token_mint.key();
        pool.pool_account = ctx.accounts.pool_account.key();
        pool.last_update_at = Clock::get()?.unix_timestamp.try_into().unwrap(); 
        pool.reward_rate = reward_rate;
        pool.period_finish = end_at;
        pool.bump = ctx.bumps.pool;
        Ok(())
    }

    /// Deposits tokens into the genesis pool.
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let user_info = &mut ctx.accounts.user_info;
        
        // Update current rewards.
        assert!(pool.update_rewards(user_info).is_ok());

        // Transfer assets in.
        let xfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            SplTransfer {
                from: ctx.accounts.user_account.to_account_info(),
                to: ctx.accounts.pool_account.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info()
            }
        );
        if !token::transfer(xfer_ctx, amount).is_ok() {
           return err!(errors::GenesisError::TokenTransferFailed); 
        }

        // Add deposit to the pool.
        pool.total_shares += amount;
        user_info.shares += amount;

        emit!(DepositEvent {
            user: ctx.accounts.depositor.key(),
            pool: pool.key(),
            amount
        });
        Ok(())
    }

    /// Withdraws tokens from the genesis pool.
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let user_info = &mut ctx.accounts.user_info;

        // Update current rewards.
        assert!(pool.update_rewards(user_info).is_ok());

        // Check if user can withdraw and update balances.
        if amount > user_info.shares {
            return err!(errors::GenesisError::WithdrawTooMuch);
        }
        user_info.shares -= amount;

        // Send tokens back to the user.
        let xfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            SplTransfer {
                from: ctx.accounts.pool_account.to_account_info(),
                to: ctx.accounts.user_account.to_account_info(),
                authority: ctx.accounts.vault.to_account_info()
            }
        );
        if !token::transfer(xfer_ctx, amount).is_ok() {
            return err!(errors::GenesisError::TokenTransferFailed);
        }

        emit!(WithdrawEvent {
            user: ctx.accounts.depositor.key(),
            pool: pool.key(),
            amount
        });
        Ok(())
    }

    /// Claims rewards from the genesis pool.
    pub fn claim_rewards(ctx: Context<ClaimRewards>) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        let user_info = &mut ctx.accounts.user_info;

        // Update rewards.
        assert!(pool.update_rewards(user_info).is_ok());
        
        // Clear any current pending rewards and send them to the user.
        let rewards = user_info.rewards;
        user_info.rewards = 0;
        if rewards > 0 {
            let mint_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: ctx.accounts.vault.to_account_info(),
                    to: ctx.accounts.user_reward_account.to_account_info(),
                    mint: ctx.accounts.reward_mint.to_account_info()
                }
            );
            if !token::mint_to(mint_ctx, rewards).is_ok() {
                return err!(errors::GenesisError::MintRewardFailed);
            };

            emit!(RewardsPaidEvent {
                user: ctx.accounts.depositor.key(),
                pool: pool.key(),
                amount: rewards
            })
        }

        Ok(())
    }
}
