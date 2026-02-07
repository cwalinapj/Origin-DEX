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
          { name: "tokenBPriceCents", type: "u64" },
          { name: "tokenAKind", type: "u8" },
          { name: "tokenBKind", type: "u8" },
          { name: "guaranteePolicy", type: "u8" },
          { name: "allowedAssetsMask", type: "u16" },
          { name: "guaranteeMint", type: "publicKey" }
        ]
      },
      {
        name: "createLpPosition",
        accounts: [
          { name: "pool", isMut: true, isSigner: false },
          { name: "position", isMut: true, isSigner: false },
          { name: "lpMint", isMut: true, isSigner: false },
          { name: "ownerLpTokenAccount", isMut: true, isSigner: false },
          { name: "owner", isMut: true, isSigner: true },
          { name: "tokenProgram", isMut: false, isSigner: false },
          { name: "associatedTokenProgram", isMut: false, isSigner: false },
          { name: "systemProgram", isMut: false, isSigner: false },
          { name: "rent", isMut: false, isSigner: false }
        ],
        args: [
          { name: "minPriceCents", type: "u64" },
          { name: "maxPriceCents", type: "u64" },
          { name: "leftFunctionType", type: "u8" },
          { name: "leftParams", type: { array: ["i64", 5] } },
          { name: "rightFunctionType", type: "u8" },
          { name: "rightParams", type: { array: ["i64", 5] } },
          { name: "amountA", type: "u64" },
          { name: "amountB", type: "u64" }
        ]
      },
      {
        name: "stakeLpNft",
        accounts: [
          { name: "pool", isMut: false, isSigner: false },
          { name: "position", isMut: false, isSigner: false },
          { name: "stake", isMut: true, isSigner: false },
          { name: "stakeVault", isMut: true, isSigner: false },
          { name: "ownerLpTokenAccount", isMut: true, isSigner: false },
          { name: "lpMint", isMut: false, isSigner: false },
          { name: "owner", isMut: true, isSigner: true },
          { name: "tokenProgram", isMut: false, isSigner: false },
          { name: "associatedTokenProgram", isMut: false, isSigner: false },
          { name: "systemProgram", isMut: false, isSigner: false },
          { name: "rent", isMut: false, isSigner: false }
        ],
        args: []
      },
      {
        name: "unstakeLpNft",
        accounts: [
          { name: "pool", isMut: false, isSigner: false },
          { name: "position", isMut: false, isSigner: false },
          { name: "stake", isMut: true, isSigner: false },
          { name: "stakeVault", isMut: true, isSigner: false },
          { name: "ownerLpTokenAccount", isMut: true, isSigner: false },
          { name: "lpMint", isMut: false, isSigner: false },
          { name: "owner", isMut: true, isSigner: true },
          { name: "tokenProgram", isMut: false, isSigner: false }
        ],
        args: []
      },
      {
        name: "addLiquidityToPosition",
        accounts: [
          { name: "pool", isMut: true, isSigner: false },
          { name: "position", isMut: true, isSigner: false },
          { name: "lpMint", isMut: true, isSigner: false },
          { name: "ownerLpTokenAccount", isMut: true, isSigner: false },
          { name: "owner", isMut: true, isSigner: true },
          { name: "tokenProgram", isMut: false, isSigner: false }
        ],
        args: [
          { name: "amountA", type: "u64" },
          { name: "amountB", type: "u64" }
        ]
      },
      {
        name: "closePosition",
        accounts: [
          { name: "pool", isMut: false, isSigner: false },
          { name: "position", isMut: true, isSigner: false },
          { name: "lpMint", isMut: true, isSigner: false },
          { name: "ownerLpTokenAccount", isMut: true, isSigner: false },
          { name: "stake", isMut: true, isSigner: false },
          { name: "owner", isMut: true, isSigner: true },
          { name: "tokenProgram", isMut: false, isSigner: false }
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
    if (data.length < 8 + 8 + 32 + 32 + 32 + 1 + 1 + 1 + 1 + 2 + 2 + 2 + 8 + 1 + 2 + 32 + 8 + 8 + 8 + 8 + 8 + 1) {
      throw new Error("Pool data too short");
    }
    const poolId = Number(data.readBigUInt64LE(8));
    const creator = new PublicKey(data.slice(8 + 8, 8 + 8 + 32));
    const tokenAMint = new PublicKey(data.slice(8 + 8 + 32, 8 + 8 + 32 + 32));
    const tokenBMint = new PublicKey(
      data.slice(8 + 8 + 32 + 32, 8 + 8 + 32 + 32 + 32)
    );
    const tokenAKind = data.readUInt8(8 + 8 + 32 + 32 + 32);
    const tokenBKind = data.readUInt8(8 + 8 + 32 + 32 + 32 + 1);
    const tokenAFrozen = data.readUInt8(8 + 8 + 32 + 32 + 32 + 2) === 1;
    const tokenBFrozen = data.readUInt8(8 + 8 + 32 + 32 + 32 + 3) === 1;
    const feeBps = data.readUInt16LE(8 + 8 + 32 + 32 + 32 + 4);
    const lpFeeBps = data.readUInt16LE(8 + 8 + 32 + 32 + 32 + 6);
    const houseFeeBps = data.readUInt16LE(8 + 8 + 32 + 32 + 32 + 8);
    const binSpacingMilliCents = Number(
      data.readBigUInt64LE(8 + 8 + 32 + 32 + 32 + 10)
    );
    const guaranteePolicy = data.readUInt8(8 + 8 + 32 + 32 + 32 + 18);
    const allowedAssetsMask = data.readUInt16LE(8 + 8 + 32 + 32 + 32 + 19);
    const guaranteeMint = new PublicKey(
      data.slice(8 + 8 + 32 + 32 + 32 + 21, 8 + 8 + 32 + 32 + 32 + 53)
    );
    const tokenAPriceCents = Number(
      data.readBigUInt64LE(8 + 8 + 32 + 32 + 32 + 53)
    );
    const tokenBPriceCents = Number(
      data.readBigUInt64LE(8 + 8 + 32 + 32 + 32 + 61)
    );
    const totalAAmount = Number(
      data.readBigUInt64LE(8 + 8 + 32 + 32 + 32 + 69)
    );
    const totalBAmount = Number(
      data.readBigUInt64LE(8 + 8 + 32 + 32 + 32 + 77)
    );
    const nextPositionId = Number(
      data.readBigUInt64LE(8 + 8 + 32 + 32 + 32 + 85)
    );
    const bump = data.readUInt8(8 + 8 + 32 + 32 + 32 + 93);
    return {
      poolId,
      creator,
      tokenAMint,
      tokenBMint,
      tokenAKind,
      tokenBKind,
      tokenAFrozen,
      tokenBFrozen,
      feeBps,
      lpFeeBps,
      houseFeeBps,
      binSpacingMilliCents,
      guaranteePolicy,
      allowedAssetsMask,
      guaranteeMint,
      tokenAPriceCents,
      tokenBPriceCents,
      totalAAmount,
      totalBAmount,
      nextPositionId,
      bump
    };
  };

  const decodePosition = (data: Buffer) => {
    if (data.length < 32 + 32 + 8 + 32 + 8 + 8 + 1 + 1 + (8 * 5) + (8 * 5) + 8 + 8 + 1) {
      throw new Error("Position data too short");
    }
    const pool = new PublicKey(data.slice(8, 8 + 32));
    const owner = new PublicKey(data.slice(8 + 32, 8 + 32 + 32));
    const positionId = Number(data.readBigUInt64LE(8 + 32 + 32));
    const lpMint = new PublicKey(
      data.slice(8 + 32 + 32 + 8, 8 + 32 + 32 + 8 + 32)
    );
    const minPriceCents = Number(
      data.readBigUInt64LE(8 + 32 + 32 + 8 + 32)
    );
    const maxPriceCents = Number(
      data.readBigUInt64LE(8 + 32 + 32 + 8 + 32 + 8)
    );
    const leftFunctionType = data.readUInt8(8 + 32 + 32 + 8 + 32 + 16);
    const rightFunctionType = data.readUInt8(8 + 32 + 32 + 8 + 32 + 17);

    const leftParams: bigint[] = [];
    const rightParams: bigint[] = [];
    let offset = 8 + 32 + 32 + 8 + 32 + 18;
    for (let i = 0; i < 5; i += 1) {
      leftParams.push(data.readBigInt64LE(offset));
      offset += 8;
    }
    for (let i = 0; i < 5; i += 1) {
      rightParams.push(data.readBigInt64LE(offset));
      offset += 8;
    }
    const amountA = Number(data.readBigUInt64LE(offset));
    offset += 8;
    const amountB = Number(data.readBigUInt64LE(offset));
    offset += 8;
    const bump = data.readUInt8(offset);

    return {
      pool,
      owner,
      positionId,
      lpMint,
      minPriceCents,
      maxPriceCents,
      leftFunctionType,
      rightFunctionType,
      leftParams,
      rightParams,
      amountA,
      amountB,
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
    if (!tokenAFrozen) {
      // USDC must be frozen; set ORIGIN_DEX_TOKEN_A_MINT to a frozen mint.
      return;
    }

    const poolBefore = await provider.connection.getAccountInfo(pool);
    if (!poolBefore) {
      const guaranteePolicy = 1; // user choice
      const allowedAssetsMask = 0b11; // WSOL + USDC
      const guaranteeMint = PublicKey.default;
      const tokenAKind = 4; // USDC
      const tokenBKind = 3; // WSOL

      await program.methods
        .createPool(
          100,
          new anchor.BN(100),
          new anchor.BN(100),
          tokenAKind,
          tokenBKind,
          guaranteePolicy,
          allowedAssetsMask,
          guaranteeMint
        )
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
    expect(poolParsed.tokenAKind).to.equal(4);
    expect(poolParsed.tokenBKind).to.equal(3);
    expect(poolParsed.tokenAFrozen).to.equal(true);
    expect(poolParsed.tokenBFrozen).to.equal(false);
    expect(poolParsed.feeBps).to.equal(100);
    expect(poolParsed.houseFeeBps).to.equal(5);
    expect(poolParsed.lpFeeBps).to.equal(95);
    expect(poolParsed.binSpacingMilliCents).to.equal(1000);
    expect(poolParsed.guaranteePolicy).to.equal(1);
    expect(poolParsed.allowedAssetsMask).to.equal(0b11);
    expect(poolParsed.guaranteeMint.toBase58()).to.equal(
      PublicKey.default.toBase58()
    );
    expect(poolParsed.tokenAPriceCents).to.equal(100);
    expect(poolParsed.tokenBPriceCents).to.equal(100);
    expect(poolParsed.totalAAmount).to.equal(0);
    expect(poolParsed.totalBAmount).to.equal(0);
    expect(poolParsed.nextPositionId).to.equal(0);

    const registryAfter = await provider.connection.getAccountInfo(registry);
    expect(registryAfter).to.not.equal(null);
    const registryParsedAfter = decodeRegistry(registryAfter!.data);
    expect(registryParsedAfter.nextPoolId).to.equal(
      registryParsed.nextPoolId + 1
    );
  });

  it("mints, stakes, and unstakes an LP NFT", async () => {
    const [registry] = PublicKey.findProgramAddressSync(
      [Buffer.from("registry")],
      programId
    );
    const regAfter = await provider.connection.getAccountInfo(registry);
    if (!regAfter) {
      return;
    }
    const data = regAfter.data;
    const nextPoolId = Number(data.readBigUInt64LE(8 + 32 + 1));
    if (nextPoolId === 0) {
      return;
    }

    const lastPoolId = nextPoolId - 1;
    const poolSeed = Buffer.alloc(8);
    poolSeed.writeBigUInt64LE(BigInt(lastPoolId));
    const [pool] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool"), poolSeed],
      programId
    );

    const poolInfo = await provider.connection.getAccountInfo(pool);
    if (!poolInfo) {
      return;
    }
    const poolParsed = decodePool(poolInfo.data);

    const positionSeed = Buffer.alloc(8);
    positionSeed.writeBigUInt64LE(BigInt(poolParsed.nextPositionId));
    const [position] = PublicKey.findProgramAddressSync(
      [Buffer.from("position"), pool.toBuffer(), positionSeed],
      programId
    );
    const [lpMint] = PublicKey.findProgramAddressSync(
      [Buffer.from("lp_mint"), position.toBuffer()],
      programId
    );

    const minPriceCents = new anchor.BN(90);
    const maxPriceCents = new anchor.BN(110);
    const leftFunctionType = 1; // linear
    const rightFunctionType = 2; // log
    const scale = new anchor.BN(1_000_000);
    const leftParams = [
      new anchor.BN(1).mul(scale),
      new anchor.BN(100).mul(scale),
      new anchor.BN(0),
      new anchor.BN(0),
      new anchor.BN(0)
    ];
    const rightParams = [
      new anchor.BN(1).mul(scale),
      new anchor.BN(10).mul(scale),
      new anchor.BN(1).mul(scale),
      new anchor.BN(100).mul(scale),
      new anchor.BN(0)
    ];

    await program.methods
      .createLpPosition(
        minPriceCents,
        maxPriceCents,
        leftFunctionType,
        leftParams,
        rightFunctionType,
        rightParams,
        new anchor.BN(10),
        new anchor.BN(10)
      )
      .accounts({
        pool,
        position,
        lpMint,
        ownerLpTokenAccount: anchor.utils.token.associatedAddress({
          mint: lpMint,
          owner: provider.wallet.publicKey
        }),
        owner: provider.wallet.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY
      })
      .rpc();

    const positionInfo = await provider.connection.getAccountInfo(position);
    expect(positionInfo).to.not.equal(null);
    const parsedPosition = decodePosition(positionInfo!.data);
    expect(parsedPosition.minPriceCents).to.equal(90);
    expect(parsedPosition.maxPriceCents).to.equal(110);
    expect(parsedPosition.leftFunctionType).to.equal(1);
    expect(parsedPosition.rightFunctionType).to.equal(2);
    expect(parsedPosition.amountA).to.equal(10);
    expect(parsedPosition.amountB).to.equal(10);

    const [stake] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake"), position.toBuffer()],
      programId
    );
    await program.methods
      .stakeLpNft()
      .accounts({
        pool,
        position,
        stake,
        stakeVault: anchor.utils.token.associatedAddress({
          mint: lpMint,
          owner: stake
        }),
        ownerLpTokenAccount: anchor.utils.token.associatedAddress({
          mint: lpMint,
          owner: provider.wallet.publicKey
        }),
        lpMint,
        owner: provider.wallet.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY
      })
      .rpc();

    await program.methods
      .unstakeLpNft()
      .accounts({
        pool,
        position,
        stake,
        stakeVault: anchor.utils.token.associatedAddress({
          mint: lpMint,
          owner: stake
        }),
        ownerLpTokenAccount: anchor.utils.token.associatedAddress({
          mint: lpMint,
          owner: provider.wallet.publicKey
        }),
        lpMint,
        owner: provider.wallet.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID
      })
      .rpc();

    await program.methods
      .addLiquidityToPosition(new anchor.BN(5), new anchor.BN(5))
      .accounts({
        pool,
        position,
        lpMint,
        ownerLpTokenAccount: anchor.utils.token.associatedAddress({
          mint: lpMint,
          owner: provider.wallet.publicKey
        }),
        owner: provider.wallet.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID
      })
      .rpc();

    const poolAfterLiquidity = await provider.connection.getAccountInfo(pool);
    expect(poolAfterLiquidity).to.not.equal(null);
    const parsedPoolAfter = decodePool(poolAfterLiquidity!.data);
    expect(parsedPoolAfter.totalAAmount).to.equal(15);
    expect(parsedPoolAfter.totalBAmount).to.equal(15);

    await program.methods
      .closePosition()
      .accounts({
        pool,
        position,
        lpMint,
        ownerLpTokenAccount: anchor.utils.token.associatedAddress({
          mint: lpMint,
          owner: provider.wallet.publicKey
        }),
        stake,
        owner: provider.wallet.publicKey,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID
      })
      .rpc();
  });
});
