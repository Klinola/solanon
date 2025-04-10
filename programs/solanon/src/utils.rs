use anchor_lang::prelude::*;
use solana_program::hash::{hashv, Hasher};

/// Generates the commitment hash by combining the deposit amount and a secret.
/// Returns a 32-byte array representing the hash.
pub fn hash_commitment(amount: u64, secret: &[u8]) -> [u8; 32] {
    let mut hasher = Hasher::default();
    // Hash the little-endian representation of the amount
    hasher.hash(&amount.to_le_bytes());
    // Hash the secret data
    hasher.hash(secret);
    // Return the resulting hash as a 32-byte array
    hasher.result().to_bytes()
}

/// Verifies a simplified Merkle proof.
///
/// # Parameters
/// - `root`: The current Merkle tree root.
/// - `proof`: The proof data, consisting of 32-byte chunks representing sibling hashes.
///            (Note: a real implementation would also need path indices.)
/// - `nullifier`: The 32-byte nullifier to be verified.
///
/// # Returns
/// Returns `true` if the derived hash from the proof matches the provided Merkle root,
/// otherwise returns `false`.
pub fn verify_merkle_proof(root: &[u8; 32], proof: &[u8], nullifier: &[u8; 32]) -> bool {
    // Compute the initial hash value from the nullifier.
    let mut current = hashv(&[nullifier]);
    // Iterate over each 32-byte chunk in the proof and combine it with the current hash.
    for chunk in proof.chunks(32) {
        let sibling: [u8; 32] = chunk.try_into().expect("Invalid proof length");
        current = hashv(&[&current.to_bytes(), &sibling]);
    }
    // Compare the final computed hash with the expected Merkle root.
    current.to_bytes() == *root
}
