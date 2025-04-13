import * as anchor from "@coral-xyz/anchor";
import { Program, BN } from "@coral-xyz/anchor";
import { Solanon } from "../target/types/solanon";
import { SystemProgram, Keypair, PublicKey } from "@solana/web3.js";
import assert from "assert";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";

describe("solanon", () => {
  // Use provider from environment (e.g. localnet).
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();
  console.log(`Provider wallet: ${provider.wallet.publicKey.toBase58()}`);

  const program = anchor.workspace.Solanon as Program<Solanon>;
  console.log(`Program ID: ${program.programId.toBase58()}`);

  it("Performs mix instruction with nonce-based PDA", async () => {
    // Fixed nonce for test
    const nonce: number = 42;
    // Number of outputs; here 2 outputs.
    const count = 2;

    // Array for output details: each element includes target output account and transfer amount.
    let outputDetails: { address: PublicKey; amount: BN }[] = [];
    // remainingAccounts array: first count are intermediate accounts, then count final output accounts.
    let intermediateAccounts: anchor.web3.AccountMeta[] = [];
    let outputAccounts: anchor.web3.AccountMeta[] = [];
    for (let i = 0; i < count; i++) {
      // Construct nonce and index buffers using BN's toBuffer method.
      const nonceBuffer = new BN(nonce).toBuffer("le", 8);
      const indexBuffer = new BN(i).toBuffer("le", 8);

      // Compute the PDA and bump using findProgramAddress with seeds:
      // ["intermediate", provider.wallet.publicKey, nonce, index]
      const [pda, bump] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("intermediate"),
          provider.wallet.publicKey.toBuffer(),
          nonceBuffer,
          indexBuffer,
        ],
        program.programId
        //SYSTEM_PROGRAM_ID
      );

      console.log(`Intermediate PDA for index ${i}: ${pda.toBase58()}, bump: ${bump}`);
      intermediateAccounts.push({
        pubkey: pda,
        isWritable: true,
        isSigner: false,
      });

      // Generate a new Keypair for the final output account.
      const outputKeypair = Keypair.generate();
      console.log(`Output account for index ${i}: ${outputKeypair.publicKey.toBase58()}`);

      // Request airdrop to the output account to ensure it has enough lamports (e.g. 1 SOL).
      const airdropSig = await provider.connection.requestAirdrop(
        outputKeypair.publicKey,
        1e9
      );
      await provider.connection.confirmTransaction(airdropSig);

      outputAccounts.push({
        pubkey: outputKeypair.publicKey,
        isWritable: true,
        isSigner: false,
      });

      // Set each output's transfer amount as BN (here 1,000,000 lamports).
      outputDetails.push({
        address: outputKeypair.publicKey,
        amount: new BN(1000000),
      });
    }
    console.log("Remaining Accounts (before mix instruction):");
    const remainingAccounts = intermediateAccounts.concat(outputAccounts);
    for (let i = 0; i < remainingAccounts.length; i++) {
      const accMeta = remainingAccounts[i];
      const accInfo = await provider.connection.getAccountInfo(accMeta.pubkey);
      if (accInfo) {
        console.log(
          `Index ${i}: ${accMeta.pubkey.toBase58()} | Writable: ${accMeta.isWritable} | Signer: ${accMeta.isSigner} | Owner: ${accInfo.owner.toBase58()} | Lamports: ${accInfo.lamports}`
        );
      } else {
        console.log(
          `Index ${i}: ${accMeta.pubkey.toBase58()} | (Account does not exist yet)`
        );
      }
    }

    const txSignature = await program.methods
      .mix(new BN(nonce), outputDetails)
      .accounts({
        user: provider.wallet.publicKey,
      })
      .remainingAccounts(remainingAccounts)
      .rpc();
    console.log("Transaction signature:", txSignature);

    console.log("Remaining Accounts (after mix instruction):");
    for (let i = 0; i < remainingAccounts.length; i++) {
      const accMeta = remainingAccounts[i];
      const accInfo = await provider.connection.getAccountInfo(accMeta.pubkey);
      if (accInfo) {
        console.log(
          `Index ${i}: ${accMeta.pubkey.toBase58()} | Owner: ${accInfo.owner.toBase58()} | Lamports: ${accInfo.lamports}`
        );
      }
    }

    for (let i = 0; i < count; i++) {
      const outputAccountMeta = remainingAccounts[count + i];
      const accountInfo = await provider.connection.getAccountInfo(outputAccountMeta.pubkey);
      assert.ok(accountInfo !== null, "Final output account does not exist");
      console.log(
        `Output account ${i} (${outputAccountMeta.pubkey.toBase58()}) lamports:`,
        accountInfo.lamports
      );
      assert.ok(
        accountInfo.lamports >= 1000000,
        "Output account did not receive the expected lamports"
      );
    }
  });
});
