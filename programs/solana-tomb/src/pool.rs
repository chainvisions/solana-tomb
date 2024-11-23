use anchor_lang::prelude::*;
use std::cmp::min;
use crate::{Depositor, Pool};

impl Pool {
    pub const POOL_SIZE: usize = 32 + 32 + 8 + 8 + 8 + 8 + 8 + 1;

    pub fn update_rewards(&mut self, user: &mut Depositor) -> Result<()> {
        if self.total_shares == 0 {
            return Ok(())
        }

        let block_timestamp: u64 = Clock::get()?.unix_timestamp.try_into().unwrap();
        let reward_per_token = self.reward_per_token(block_timestamp);
        let earned = self.user_earnings(user, reward_per_token);

        self.reward_per_share = reward_per_token;
        self.last_update_at = block_timestamp;
        user.rewards = earned;
        user.reward_paid_per_shares = reward_per_token;

        Ok(()) 
    }

    fn reward_per_token(&mut self, current_ts: u64) -> u64 {
        if self.total_shares == 0 {
            return self.reward_per_share;
        }

        let last_reward_applicable_at: u64 = min(current_ts, self.period_finish);
        self.reward_per_share + (((last_reward_applicable_at - self.last_update_at) * self.reward_rate) * (10^9)) / self.total_shares
    }

    fn user_earnings(&self, user: &Depositor, latest_reward_per_token: u64) -> u64 {
        let rewards: u64 = ((user.shares * (latest_reward_per_token - self.reward_per_share)) / 10^9) + user.rewards;
        rewards
    }
}
