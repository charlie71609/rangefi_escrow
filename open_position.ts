import DLMM from "@meteora-ag/dlmm";
import {
  Connection,
  Keypair,
  PublicKey,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { BN } from "bn.js";
import fs from "fs";

const connection = new Connection("https://api.devnet.solana.com", "confirmed");

const walletRaw = JSON.parse(fs.readFileSync("/Users/admin/.config/solana/id.json", "utf-8"));
const wallet = Keypair.fromSecretKey(Uint8Array.from(walletRaw));

const POOL_ADDRESS = new PublicKey("2R57whGsBceGqQrV4YbMBFRnBSAvs2PMeNqWbFrBHMXS");

async function main() {
  console.log("Wallet:", wallet.publicKey.toBase58());

  const dlmm = await DLMM.create(connection, POOL_ADDRESS);
  console.log("Pool loaded. Active bin ID:", dlmm.lbPair.activeId);

  const activeBin = await dlmm.getActiveBin();
  console.log("Active bin price:", activeBin.pricePerToken);

  const lowerBinId = activeBin.binId - 15;
  const upperBinId = activeBin.binId - 1;
  const width = upperBinId - lowerBinId + 1;

  console.log(`Opening position: bins ${lowerBinId} to ${upperBinId} (width=${width})`);

  const positionKeypair = Keypair.generate();
  console.log("Position keypair:", positionKeypair.publicKey.toBase58());

  const totalXAmount = new BN(0);
  const totalYAmount = new BN(100_000_000);

  const tx = await dlmm.initializePositionAndAddLiquidityByStrategy({
    positionPubKey: positionKeypair.publicKey,
    user: wallet.publicKey,
    totalXAmount,
    totalYAmount,
    strategy: {
      maxBinId: upperBinId,
      minBinId: lowerBinId,
      strategyType: 0,
    },
    slippage: 1,
  });

  const sig = await sendAndConfirmTransaction(connection, tx, [wallet, positionKeypair]);
  console.log("Signature:", sig);
  console.log("Position opened! Address:", positionKeypair.publicKey.toBase58());
}

main().catch(console.error);