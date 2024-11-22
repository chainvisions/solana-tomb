use anchor_lang::prelude::*;
use anchor_lang::system_program;
use instructions::*;

declare_id!("EbUS5R37dPQRdDKgh7Kju9JhwUg6FPB1fSRAjbUj29EQ");

pub mod state;
pub mod instructions;
pub mod errors;

#[program]
pub mod solana_tomb {
    use super::*;

    pub fn initialize(ctx: Context<InitializeGenesis>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.authority = ctx.accounts.authority.key();
        state.dev_share = ctx.accounts.authority.key(); // TODO: We prob need to create a new token account for this
        state.reward_token = ctx.accounts.reward_mint.key();

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
}

#[derive(Accounts)]
pub struct Initialize {}
