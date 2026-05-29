import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RangefiEscrow } from "../target/types/rangefi_escrow";
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";

describe("rangefi_escrow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.RangefiEscrow as Program<RangefiEscrow>;
  const borrower = provider.wallet;

  const ESCROW_SEED = Buffer.from("escrow");
  const ESCROW_PDA_SEED = Buffer.from("pda");
  const LB_CLMM_PROGRAM = new PublicKey("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");

  const [escrowState] = PublicKey.findProgramAddressSync(
    [ESCROW_SEED, borrower.publicKey.toBuffer()],
    program.programId
  );

  const [escrowPda] = PublicKey.findProgramAddressSync(
    [ESCROW_SEED, ESCROW_PDA_SEED, borrower.publicKey.toBuffer()],
    program.programId
  );

  // Derive event_authority once at the top
  const [eventAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from("__event_authority")],
    LB_CLMM_PROGRAM
  );

  it("Verifies existing escrow state on-chain", async () => {
    console.log("Borrower:    ", borrower.publicKey.toBase58());
    console.log("Escrow PDA:  ", escrowPda.toBase58());
    console.log("Escrow State:", escrowState.toBase58());

    const state = await program.account.escrowState.fetch(escrowState);
    console.log("Stored borrower:", state.borrower.toBase58());
    console.log("Stored position:", state.position.toBase58());
    console.log("Stored lb_pair: ", state.lbPair.toBase58());
    console.log("Escrow state verified ✅");
  });

  it("Opens position via Meteora CPI", async () => {
    const lbPair = new PublicKey("Fwwh8zZZExaG1YDBJH3re7iP24fCceFhko1e4hZg9Upv");

    const lowerBinId = -10;
    const width = 20;

    const lowerBinIdBuffer = Buffer.alloc(4);
    lowerBinIdBuffer.writeInt32LE(lowerBinId, 0);
    const widthBuffer = Buffer.alloc(4);
    widthBuffer.writeInt32LE(width, 0);

    const [position] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("position"),
        lbPair.toBuffer(),
        escrowPda.toBuffer(),
        lowerBinIdBuffer,
        widthBuffer,
      ],
      LB_CLMM_PROGRAM
    );

    console.log("Position PDA:", position.toBase58());
    console.log("lb_pair:     ", lbPair.toBase58());
    console.log("event_authority:", eventAuthority.toBase58());

    const tx = await program.methods
      .escrowOpen(lowerBinId, width)
      .accounts({
        borrower: borrower.publicKey,
        escrowPda,
        escrowState,
        position,
        lbPair,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
        eventAuthority,
        lbClmmProgram: LB_CLMM_PROGRAM,
      })
      .rpc();

    console.log("Transaction:", tx);
    console.log("Solscan: https://solscan.io/tx/" + tx + "?cluster=devnet");
    console.log("Position owner should be escrow PDA ✅");
  });
});