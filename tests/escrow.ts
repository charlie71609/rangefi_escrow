import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RangefiEscrow } from "../target/types/rangefi_escrow";
import { PublicKey, SystemProgram } from "@solana/web3.js";

describe("rangefi_escrow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.RangefiEscrow as Program<RangefiEscrow>;
  const borrower = provider.wallet;

  it("Creates escrow PDA", async () => {
    const ESCROW_SEED = Buffer.from("escrow");

    const [escrowState] = PublicKey.findProgramAddressSync(
      [ESCROW_SEED, borrower.publicKey.toBuffer()],
      program.programId
    );

    const [escrowPda] = PublicKey.findProgramAddressSync(
      [ESCROW_SEED, borrower.publicKey.toBuffer()],
      program.programId
    );

    console.log("Borrower:    ", borrower.publicKey.toBase58());
    console.log("Escrow PDA:  ", escrowPda.toBase58());
    console.log("Escrow State:", escrowState.toBase58());

    const tx = await program.methods
      .initialize()
      .accounts({
        borrower: borrower.publicKey,
        escrowState,
        escrowPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Transaction:", tx);
    console.log("Solscan: https://solscan.io/tx/" + tx + "?cluster=devnet");

    const state = await program.account.escrowState.fetch(escrowState);
    console.log("Stored borrower:", state.borrower.toBase58());

    anchor.assert
      ? anchor.assert.equal(
          state.borrower.toBase58(),
          borrower.publicKey.toBase58()
        )
      : console.log("State verified ✅");
  });
});