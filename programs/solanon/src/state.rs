use anchor_lang::prelude::*;

/// Constant for the mixer state account size.
/// Adjust this value according to the actual space required.
pub const MIXER_STATE_SIZE: usize = 1024; // Example size, adjust as needed

#[account]
pub struct MixerState {
    /// The Merkle tree root.
    pub root: [u8; 32],
    /// List of commitments (each is a 32-byte hash).
    pub commitments: Vec<[u8; 32]>,
    /// List of spent nullifiers.
    pub nullifiers: Vec<[u8; 32]>,
}

impl MixerState {
    /// Adds a new commitment to the state.
    /// This is a simplified version; in a real implementation,
    /// you should recalculate the Merkle tree and update the root properly.
    pub fn add_commitment(&mut self, commitment: [u8; 32]) -> Result<()> {
        self.commitments.push(commitment);
        // Simplified: update the root with the new commitment.
        self.root = commitment;
        Ok(())
    }

    /// Marks a nullifier as spent by appending it to the nullifiers list.
    pub fn mark_nullifier_spent(&mut self, nullifier: [u8; 32]) -> Result<()> {
        self.nullifiers.push(nullifier);
        Ok(())
    }

    /// Checks whether the given nullifier has already been spent.
    pub fn is_nullifier_spent(&self, nullifier: &[u8; 32]) -> bool {
        self.nullifiers.contains(nullifier)
    }
}
