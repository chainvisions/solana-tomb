use anchor_lang::prelude::*;

#[error_code]
pub enum GenesisError {
    #[msg("Withdrawal exceeds staked amount")]
    WithdrawTooMuch,
    #[msg("SOL transfer failed")]
    SolTransferFailed,
    #[msg("Token transfer failed")]
    TokenTransferFailed,
    #[msg("Minting rewards failed")]
    MintRewardFailed
}
