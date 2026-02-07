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
      },
      {
        name: "initRegistry",
        accounts: [
          { name: "registry", isMut: true, isSigner: false },
          { name: "admin", isMut: true, isSigner: true },
          { name: "systemProgram", isMut: false, isSigner: false }
        ],
        args: []
      },
      {
        name: "createPool",
        accounts: [
          { name: "registry", isMut: true, isSigner: false },
          { name: "pool", isMut: true, isSigner: false },
          { name: "tokenAMint", isMut: false, isSigner: false },
          { name: "tokenBMint", isMut: false, isSigner: false },
          { name: "admin", isMut: true, isSigner: true },
          { name: "systemProgram", isMut: false, isSigner: false }
        ],
        args: [
          { name: "feeBps", type: "u16" },
          { name: "tokenAPriceCents", type: "u64" },
          { name: "tokenBPriceCents", type: "u64" }
        ]
      }
    ]
  } as anchor.Idl;

  const program = new anchor.Program(idl, programId, provider);

  const decodeConfig = (data: Buffer) => {
    if (data.length < 8 + 32 + 1 + 1) {
      throw new Error("Config data too short");
    }
    const admin = new PublicKey(data.slice(8, 8 + 32));
    const bump = data.readUInt8(8 + 32);
    const initialized = data.readUInt8(8 + 32 + 1) === 1;
    return { admin, bump, initialized };
  };

  const decodeRegistry = (data: Buffer) => {
    if (data.length < 8 + 32 + 1 + 8 + 1) {
      throw new Error("Registry data too short");
    }
    const admin = new PublicKey(data.slice(8, 8 + 32));
    const bump = data.readUInt8(8 + 32);
    const nextPoolId = Number(data.readBigUInt64LE(8 + 32 + 1));
    const initialized = data.readUInt8(8 + 32 + 1 + 8) === 1;
    return { admin, bump, nextPoolId, initialized };
  };

  const decodePool = (data: Buffer) => {
    if (data.length < 8 + 8 + 32 + 32 + 32 + 1 + 1 + 2 + 2 + 2 + 8 + 1) {
      throw new Error("Pool data too short");
    }
    const poolId = Number(data.readBigUInt64LE(8));
    const creator = new PublicKey(data.slice(8 + 8, 8 + 8 + 32));
    const tokenAMint = new PublicKey(data.slice(8 + 8 + 32, 8 + 8 + 32 + 32));
    const tokenBMint = new PublicKey(
      data.slice(8 + 8 + 32 + 32, 8 + 8 + 32 + 32 + 32)
    );
    const tokenAFrozen = data.readUInt8(8 + 8 + 32 + 32 + 32) === 1;
    const tokenBFrozen = data.readUInt8(8 + 8 + 32 + 32 + 32 + 1) === 1;
    const feeBps = data.readUInt16LE(8 + 8 + 32 + 32 + 32 + 2);
    const lpFeeBps = data.readUInt16LE(8 + 8 + 32 + 32 + 32 + 4);
    const houseFeeBps = data.readUInt16LE(8 + 8 + 32 + 32 + 32 + 6);
    const binSpacingMilliCents = Number(
      data.readBigUInt64LE(8 + 8 + 32 + 32 + 32 + 8)
    );
    const bump = data.readUInt8(8 + 8 + 32 + 32 + 32 + 16);
    return {
      poolId,
      creator,
      tokenAMint,
      tokenBMint,
      tokenAFrozen,
      tokenBFrozen,
      feeBps,
      lpFeeBps,
      houseFeeBps,
      binSpacingMilliCents,
      bump
    };
  };

  it("initializes config PDA and validates data", async () => {
    const [config] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      programId
    );

    const before = await provider.connection.getAccountInfo(config);
    if (before) {
      const parsed = decodeConfig(before.data);
      expect(parsed.admin.toBase58()).to.equal(
        provider.wallet.publicKey.toBase58()
      );
      expect(parsed.initialized).to.equal(true);
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
    const parsed = decodeConfig(after!.data);
    expect(parsed.admin.toBase58()).to.equal(
      provider.wallet.publicKey.toBase58()
    );
    expect(parsed.initialized).to.equal(true);
  });

  it("initializes registry and creates a pool", async () => {
    const [registry] = PublicKey.findProgramAddressSync(
      [Buffer.from("registry")],
      programId
    );

    let registryAccount = await provider.connection.getAccountInfo(registry);
    if (!registryAccount) {
      await program.methods
        .initRegistry()
        .accounts({
          registry,
          admin: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId
        })
        .rpc();
      registryAccount = await provider.connection.getAccountInfo(registry);
    }

    expect(registryAccount).to.not.equal(null);
    const registryParsed = decodeRegistry(registryAccount!.data);
    expect(registryParsed.admin.toBase58()).to.equal(
      provider.wallet.publicKey.toBase58()
    );
    expect(registryParsed.initialized).to.equal(true);

    const poolId = BigInt(registryParsed.nextPoolId);
    const poolIdBytes = Buffer.alloc(8);
    poolIdBytes.writeBigUInt64LE(poolId);
    const [pool] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), poolIdBytes],
      programId
    );

    const tokenAMint = new PublicKey(
      process.env.ORIGIN_DEX_TOKEN_A_MINT ||
        "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"
    );
    const tokenBMint = new PublicKey(
      process.env.ORIGIN_DEX_TOKEN_B_MINT ||
        "So11111111111111111111111111111111111111112"
    );

    const tokenAInfo = await provider.connection.getParsedAccountInfo(tokenAMint);
    const tokenBInfo = await provider.connection.getParsedAccountInfo(tokenBMint);
    const tokenAFrozen =
      tokenAInfo.value?.data &&
      "parsed" in tokenAInfo.value.data &&
      tokenAInfo.value.data.parsed.info.freezeAuthority;
    const tokenBFrozen =
      tokenBInfo.value?.data &&
      "parsed" in tokenBInfo.value.data &&
      tokenBInfo.value.data.parsed.info.freezeAuthority;
    if (!tokenAFrozen || !tokenBFrozen) {
      // Skip if default mints are not frozen; set env vars to frozen mints to test.
      return;
    }

    const poolBefore = await provider.connection.getAccountInfo(pool);
    if (!poolBefore) {
      await program.methods
        .createPool(100, new anchor.BN(100), new anchor.BN(100))
        .accounts({
          registry,
          pool,
          tokenAMint,
          tokenBMint,
          admin: provider.wallet.publicKey,
          systemProgram: SystemProgram.programId
        })
        .rpc();
    }

    const poolAfter = await provider.connection.getAccountInfo(pool);
    expect(poolAfter).to.not.equal(null);
    const poolParsed = decodePool(poolAfter!.data);
    expect(poolParsed.poolId).to.equal(registryParsed.nextPoolId);
    expect(poolParsed.creator.toBase58()).to.equal(
      provider.wallet.publicKey.toBase58()
    );
    expect(poolParsed.tokenAMint.toBase58()).to.equal(tokenAMint.toBase58());
    expect(poolParsed.tokenBMint.toBase58()).to.equal(tokenBMint.toBase58());
    expect(poolParsed.tokenAFrozen).to.equal(true);
    expect(poolParsed.tokenBFrozen).to.equal(true);
    expect(poolParsed.feeBps).to.equal(100);
    expect(poolParsed.houseFeeBps).to.equal(5);
    expect(poolParsed.lpFeeBps).to.equal(95);
    expect(poolParsed.binSpacingMilliCents).to.equal(1000);

    const registryAfter = await provider.connection.getAccountInfo(registry);
    expect(registryAfter).to.not.equal(null);
    const registryParsedAfter = decodeRegistry(registryAfter!.data);
    expect(registryParsedAfter.nextPoolId).to.equal(
      registryParsed.nextPoolId + 1
    );
  });
});
