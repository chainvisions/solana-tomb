use anchor_lang::prelude::*;

#[account]
pub struct Genesis {
    pub authority: Pubkey,
    pub dev_share: Pubkey,
    pub reward_token: Pubkey,
    pub vault: Pubkey,
    pub bump: u8,
    pub vault_bump: u8
}

#[account]
pub struct Pool {
    pub underlying: Pubkey,
    pub pool_account: Pubkey,
    pub total_shares: u64,
    pub reward_rate: u64,
    pub reward_per_share: u64,
    pub is_init: bool,
    pub bump: u8
}

#[account]
pub struct Depositor {
    pub authority: Pubkey,
    pub shares: u64,
    pub reward_paid_per_shares: u64,
    pub is_init: bool,
    pub bump: u8
}

impl Genesis {
    pub const STATE_SIZE: usize = 32 + 32 + 32 + 32 + 1 + 1;
}

impl Pool {
    pub const POOL_SIZE: usize = 32 + 32 + 8 + 8 + 8 + 1 + 1;
}

impl Depositor {
    pub const DEPOSITOR_SIZE: usize = 32 + 8 + 8 + 1 + 1;
}
