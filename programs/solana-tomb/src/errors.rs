use anchor_lang::prelude::*;

#[error_code]
pub enum GenesisError {
    #[msg("SOL transfer failed")]
    SolTransferFailed
}
