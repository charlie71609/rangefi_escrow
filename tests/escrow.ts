import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { RangefiEscrow } from "../target/types/rangefi_escrow";
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from "@solana/web3.js";
import * as fs from "fs";

describe("rangefi_escrow — precise deposit fidelity", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.RangefiEscrow as Program<RangefiEscrow>;
  const borrower = provider.wallet;

  const ESCROW_SEED = Buffer.from("escrow");
  const ESCROW_PDA_SEED = Buffer.from("pda");
  const LB_CLMM_PROGRAM = new PublicKey("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");

  const BORROWER_POSITION = new PublicKey("EkgwqVCviE9Lk1mff2Z19hJQP9S3MdMFVUKtd8tML8PM");
  const ESCROW_POSITION = new PublicKey("ZQHV5P4KAPTcWJjVFX2sWZaM8mUr1s9V8he4FnQZQmv");

  // NEW: fresh empty borrower position created by open_borrower_release_position.js
  const BORROWER_RELEASE_POSITION = new PublicKey("BAKjCok3YQTMMbPewZ8P6RMetCinZuAKBoxPWnvCvFSW");
  // NEW: escrow USDC ATA — remove's X-side destination (0 removed, layout-required)
  const escrowTokenX = new PublicKey("AYDERxiWjxC96nQTYasPCdrkzeEYooGfYBaCVJ4kzUoo");

  const LOWER_BIN_ID = -12;
  const WIDTH = 15;

  const lbPair          = new PublicKey("2R57whGsBceGqQrV4YbMBFRnBSAvs2PMeNqWbFrBHMXS");
  const reserveX        = new PublicKey("JUWmYjqmpHtRAKwtymcfoXHoxPysi1AsMa7cwsmNq13");
  const reserveY        = new PublicKey("92u4Ngq7UsssQWHNoarFTkdiseGBs2QVkwzsN5NA1nh");
  const tokenXMint      = new PublicKey("Gh9ZwEmdLJ8DscKNTkTqPbNwLNNBjuSzaG9Vp2KGtKJr");
  const tokenYMint      = new PublicKey("So11111111111111111111111111111111111111112");
  const binArrayLower   = new PublicKey("7ABcgNrHEcAvox4x2gUbvtadD5RWmaoA1LDeeGNLuan2");
  const binArrayUpper   = new PublicKey("Fdy2WDHER7uoXfzxhhwuXSCZLF6KUuXSum6CfN3hT1HY");
  const TOKEN_PROGRAM   = new PublicKey("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
  const userTokenY      = new PublicKey("FyCgYkv7zQPUmx55MqiSCb7yjMcYL9tSeVZwF2EYHwty");
  const userTokenX      = new PublicKey("A2WPQLbQzpZGeUxJjMSy4jCAjijm7N2zhGrHFWUKPDzN");
  const escrowTokenY    = new PublicKey("CZyAVrVRiwGjhEvpkj37THdgSZf4kHsdXZMUT7qfU6Jq");
  const binArrayBitmapExt = LB_CLMM_PROGRAM;

  const [escrowState] = PublicKey.findProgramAddressSync(
    [ESCROW_SEED, borrower.publicKey.toBuffer()],
    program.programId
  );
  const [escrowPda] = PublicKey.findProgramAddressSync(
    [ESCROW_SEED, ESCROW_PDA_SEED, borrower.publicKey.toBuffer()],
    program.programId
  );
  const [eventAuthority] = PublicKey.findProgramAddressSync(
    [Buffer.from("__event_authority")],
    LB_CLMM_PROGRAM
  );

  it("Opens a fresh escrow position at bins -10 to 4", async () => {
    const tx = await program.methods
      .escrowOpen(LOWER_BIN_ID, WIDTH)
      .accounts({
        borrower: borrower.publicKey,
        escrowPda,
        escrowState,
        position: ESCROW_POSITION,
        lbPair,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
        eventAuthority,
        lbClmmProgram: LB_CLMM_PROGRAM,
      })
      .rpc();
    console.log("escrow_open tx:", tx);
  });

  it("Removes liquidity from borrower position", async () => {
    const binLiquidityReductions = [];
    for (let binId = LOWER_BIN_ID; binId <= LOWER_BIN_ID + WIDTH - 1; binId++) {
      binLiquidityReductions.push({ binId, bpsToRemove: 10000 });
    }
    const tx = await program.methods
      .removeCollateral(binLiquidityReductions)
      .accounts({
        borrower: borrower.publicKey,
        position: BORROWER_POSITION,
        lbPair,
        binArrayBitmapExt,
        userTokenX,
        userTokenY,
        reserveX,
        reserveY,
        tokenXMint,
        tokenYMint,
        binArrayLower,
        binArrayUpper,
        tokenXProgram: TOKEN_PROGRAM,
        tokenYProgram: TOKEN_PROGRAM,
        eventAuthority,
        lbClmmProgram: LB_CLMM_PROGRAM,
      })
      .rpc();
    console.log("remove tx:", tx);
  });

  it("Deposits collateral with EXACT per-bin amounts (precise)", async () => {
    const baseline = JSON.parse(fs.readFileSync("borrower_baseline_curve.json", "utf-8"));
    const bins = baseline.bins.map((b: any) => ({
      binId: b.binId,
      amount: Number(b.y),
    }));
    const liquidityParameter = {
      bins,
      decompressMultiplier: new anchor.BN(1),
    };
    const tx = await program.methods
      .depositCollateral([], liquidityParameter)
      .accounts({
        borrower: borrower.publicKey,
        escrowPda,
        escrowState,
        position: BORROWER_POSITION,
        escrowPosition: ESCROW_POSITION,
        lbPair,
        binArrayBitmapExt,
        userTokenX,
        userTokenY,
        reserveX,
        reserveY,
        tokenXMint,
        tokenYMint,
        binArrayLower,
        binArrayUpper,
        tokenXProgram: TOKEN_PROGRAM,
        tokenYProgram: TOKEN_PROGRAM,
        eventAuthority,
        lbClmmProgram: LB_CLMM_PROGRAM,
        escrowTokenY,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("deposit tx:", tx);
  });

  it.only("Releases collateral: drains escrow position into fresh borrower position, closes escrow position", async () => {
    const baseline = JSON.parse(fs.readFileSync("borrower_baseline_curve.json", "utf-8"));
    const bins = baseline.bins.map((b: any) => ({
      binId: b.binId,
      amount: Number(b.y),
    }));
    const total = bins.reduce((s: number, b: any) => s + b.amount, 0);
    console.log("Per-bin amounts to return:", bins.map((b: any) => b.binId + ":" + b.amount).join("  "));
    console.log("Exact total to release:", total);

    const liquidityParameter = {
      bins,
      decompressMultiplier: new anchor.BN(1),
    };

    const borrowerBefore = await provider.connection.getTokenAccountBalance(userTokenY);
    console.log("Borrower wSOL before release:", borrowerBefore.value.amount);

    const tx = await program.methods
      .releaseCollateral(liquidityParameter)
      .accounts({
        borrower: borrower.publicKey,
        escrowPda,
        escrowState,
        escrowPosition: ESCROW_POSITION,
        borrowerPosition: BORROWER_RELEASE_POSITION,
        lbPair,
        binArrayBitmapExt,
        escrowTokenX,
        escrowTokenY,
        userTokenY,
        reserveX,
        reserveY,
        tokenXMint,
        tokenYMint,
        binArrayLower,
        binArrayUpper,
        tokenXProgram: TOKEN_PROGRAM,
        tokenYProgram: TOKEN_PROGRAM,
        eventAuthority,
        lbClmmProgram: LB_CLMM_PROGRAM,
        systemProgram: SystemProgram.programId,
      })
      .rpc();
    console.log("release tx:", tx);
    console.log("Solscan: https://solscan.io/tx/" + tx + "?cluster=devnet");
  });
});