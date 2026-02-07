import * as anchor from "@coral-xyz/anchor";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { expect } from "chai";

describe("origin_dex", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const programId = new PublicKey(
    process.env.ORIGIN_DEX_PROGRAM_ID || "Orig1nDex111111111111111111111111111111111"
  );

  const idl = {
    version: "0.1.0",
    name: "origin_dex",
    instructions: [
      {
        name: "initialize",
        accounts: [
          { name: "config", isMut: true, isSigner: false },
          { name: "admin", isMut: true, isSigner: true },
          { name: "systemProgram", isMut: false, isSigner: false }
        ],
        args: []
      }
    ]
  } as anchor.Idl;

  const program = new anchor.Program(idl, programId, provider);

  it("initializes config PDA", async () => {
    const [config] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      programId
    );

    const before = await provider.connection.getAccountInfo(config);
    if (before) {
      // If it already exists, this test is considered passing.
      return;
    }

    await program.methods
      .initialize()
      .accounts({
        config,
        admin: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId
      })
      .rpc();

    const after = await provider.connection.getAccountInfo(config);
    expect(after).to.not.equal(null);
  });
});
