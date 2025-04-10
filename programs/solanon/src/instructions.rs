use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    system_instruction,
    program::invoke,
};
use crate::{
    state::MixerState,
    utils::{hash_commitment, verify_merkle_proof},
};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Mixer state account must be owned by the current program and allocated with sufficient space.
    #[account(mut, constraint = mixer_account.owner == program_id.key())]
    pub mixer_account: Account<'info, MixerState>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub mixer_account: Account<'info, MixerState>,
}

/// Deposit instruction: transfer SOL from the payer to the mixer account, then generate and store a commitment.
/// `amount`: the deposit amount in lamports.
/// `secret`: a 32-byte secret used to generate the commitment.
pub fn deposit(ctx: Context<Deposit>, amount: u64, secret: [u8; 32]) -> Result<()> {
    // Create a transfer instruction from the payer to the mixer account.
    let transfer_ix = system_instruction::transfer(
        ctx.accounts.payer.key,
        ctx.accounts.mixer_account.to_account_info().key,
        amount,
    );

    // Invoke the system program to execute the transfer.
    invoke(
        &transfer_ix,
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.mixer_account.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Generate the commitment hash using the deposit amount and secret.
    let commitment = hash_commitment(amount, &secret);

    // Update the mixer state by adding the new commitment.
    let mixer_state = &mut ctx.accounts.mixer_account;
    mixer_state.add_commitment(commitment)?;

    // Log the successful deposit.
    msg!("Deposited {} lamports with commitment", amount);
    Ok(())
}

/// Withdraw instruction: verify the nullifier and Merkle proof, then mark the nullifier as spent.
/// `nullifier`: a 32-byte value provided by the user as proof of inclusion in the mixer.
/// `proof`: a vector of bytes representing the simplified Merkle proof.
pub fn withdraw(ctx: Context<Withdraw>, nullifier: [u8; 32], proof: Vec<u8>) -> Result<()> {
    let mixer_state = &mut ctx.accounts.mixer_account;

    // Check if the nullifier has already been spent.
    if mixer_state.is_nullifier_spent(&nullifier) {
        return Err(ErrorCode::NullifierAlreadySpent.into());
    }

    // Verify the provided Merkle proof against the current state root.
    if !verify_merkle_proof(&mixer_state.root, &proof, &nullifier) {
        return Err(ErrorCode::InvalidMerkleProof.into());
    }

    // Mark the nullifier as spent in the mixer state.
    mixer_state.mark_nullifier_spent(nullifier)?;

    // Log the successful withdrawal.
    msg!("Withdrawal processed");
    Ok(())
}

#[error_code]
pub enum ErrorCode {
    #[msg("Nullifier has already been spent")]
    NullifierAlreadySpent,
    #[msg("Invalid Merkle proof")]
    InvalidMerkleProof,
}
