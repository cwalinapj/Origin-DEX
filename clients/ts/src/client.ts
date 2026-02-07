import * as anchor from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";

const PROGRAM_ID = new PublicKey(
  process.env.ORIGIN_DEX_PROGRAM_ID || "Orig1nDex111111111111111111111111111111111"
);

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const [configPda] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    PROGRAM_ID
  );

  console.log("Program ID:", PROGRAM_ID.toBase58());
  console.log("Config PDA:", configPda.toBase58());

  const accountInfo = await provider.connection.getAccountInfo(configPda);
  if (!accountInfo) {
    console.log("Config account not found. Run initialize via Anchor/CLI first.");
    return;
  }

  console.log("Config account exists. Data length:", accountInfo.data.length);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
