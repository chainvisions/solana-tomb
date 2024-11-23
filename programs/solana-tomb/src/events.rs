use anchor_lang::prelude::*;

#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub amount: u64
}

#[event]
pub struct WithdrawEvent {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub amount: u64
}

#[event]
pub struct RewardsPaidEvent {
    pub user: Pubkey,
    pub pool: Pubkey,
    pub amount: u64
}
