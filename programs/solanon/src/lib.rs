use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod utils;
mod program_id;

declare_id!(program_id::PROGRAM_ID);

#[program]
pub mod solanon {
    use super::*;
    
    pub fn deposit(ctx: Context<instructions::Deposit>, amount: u64, secret: [u8; 32]) -> Result<()> {
        instructions::deposit(ctx, amount, secret)
    }
    
    pub fn withdraw(ctx: Context<instructions::Withdraw>, nullifier: [u8; 32], proof: Vec<u8>) -> Result<()> {
        instructions::withdraw(ctx, nullifier, proof)
    }
}
