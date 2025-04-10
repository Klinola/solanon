use anchor_lang::prelude::*;

declare_id!("EYutos9XnBjwzGUs1rz1s29f1AUwcEDWq3eaJm43pEex");

#[program]
pub mod solanon {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
