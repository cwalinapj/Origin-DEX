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
          { name: "admin", isMut: true, isSigner: true },
          { name: "systemProgram", isMut: false, isSigner: false }
        ],
        args: []
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
    if (data.length < 8 + 8 + 32 + 1) {
      throw new Error("Pool data too short");
    }
    const poolId = Number(data.readBigUInt64LE(8));
    const creator = new PublicKey(data.slice(8 + 8, 8 + 8 + 32));
    const bump = data.readUInt8(8 + 8 + 32);
    return { poolId, creator, bump };
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

    const poolBefore = await provider.connection.getAccountInfo(pool);
    if (!poolBefore) {
      await program.methods
        .createPool()
        .accounts({
          registry,
          pool,
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

    const registryAfter = await provider.connection.getAccountInfo(registry);
    expect(registryAfter).to.not.equal(null);
    const registryParsedAfter = decodeRegistry(registryAfter!.data);
    expect(registryParsedAfter.nextPoolId).to.equal(
      registryParsed.nextPoolId + 1
    );
  });
});
