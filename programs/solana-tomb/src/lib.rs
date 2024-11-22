use anchor_lang::prelude::*;

declare_id!("EbUS5R37dPQRdDKgh7Kju9JhwUg6FPB1fSRAjbUj29EQ");

#[program]
pub mod solana_tomb {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
