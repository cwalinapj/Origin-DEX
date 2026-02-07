# Devnet Readiness

This repo includes a minimal Anchor program with a config PDA. It is deployable but not a DEX. Use the steps below to deploy and validate basic connectivity on devnet.

## Solana CLI config
```bash
solana config set --url https://api.devnet.solana.com
```

## Wallet setup
The README references `scripts/setup_devnet_wallet_macos_v2.sh`. A placeholder script is provided but not implemented.

Manual flow:
```bash
solana-keygen new --outfile ./devnet-wallet.json
solana config set --keypair ./devnet-wallet.json
solana airdrop 2
```

## Anchor program deploy
```bash
anchor build
anchor deploy --provider.cluster devnet --provider.wallet ./devnet-wallet.json
```

After deploy, copy the program id into:
- `/Users/root1/scripts/Origin-DEX/Anchor.toml` for `[programs.devnet]`
- `ORIGIN_DEX_PROGRAM_ID` env var for the clients

## Initialize config PDA
This repo does not include an Anchor test script. You can initialize using the Anchor CLI in a one-off script.

Example (TypeScript):
```bash
cd /Users/root1/scripts/Origin-DEX/clients/ts
npm install
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
export ANCHOR_WALLET=/absolute/path/to/devnet-wallet.json
export ORIGIN_DEX_PROGRAM_ID=ReplaceWithProgramId
node -e "import * as anchor from '@coral-xyz/anchor';\
import { PublicKey } from '@solana/web3.js';\
const programId = new PublicKey(process.env.ORIGIN_DEX_PROGRAM_ID);\
const provider = anchor.AnchorProvider.env();\
anchor.setProvider(provider);\
const [config] = PublicKey.findProgramAddressSync([Buffer.from('config')], programId);\
const idl = { version: '0.1.0', name: 'origin_dex', instructions: [ { name: 'initialize', accounts: [ { name: 'config', isMut: true, isSigner: false }, { name: 'admin', isMut: true, isSigner: true }, { name: 'systemProgram', isMut: false, isSigner: false } ], args: [] } ] };\
const program = new anchor.Program(idl, programId, provider);\
await program.methods.initialize().accounts({ config, admin: provider.wallet.publicKey, systemProgram: anchor.web3.SystemProgram.programId }).rpc();\
console.log('Initialized', config.toBase58());"
```

## Initialize registry and create a pool
The registry tracks a monotonically increasing `next_pool_id`. `create_pool` uses the current `next_pool_id` as the pool PDA seed.
Pool creation requires **both token mints to have a freeze authority**.

Pool parameters:
- `fee_bps`: total trading fee in basis points. The house takes 5% of this fee; LPs receive the remainder.
- `token_a_price_cents` / `token_b_price_cents`: used to derive bin spacing.
  - Bin spacing is `avg_price_cents * 10` (milli-cents), so $1.00 => 1000 (1 cent), $0.50 => 500 (0.5 cents), $10.00 => 10000 (10 cents).

Example (TypeScript):
```bash
cd /Users/root1/scripts/Origin-DEX/clients/ts
npm install
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
export ANCHOR_WALLET=/absolute/path/to/devnet-wallet.json
export ORIGIN_DEX_PROGRAM_ID=ReplaceWithProgramId
export ORIGIN_DEX_TOKEN_A_MINT=FrozenMintA
export ORIGIN_DEX_TOKEN_B_MINT=FrozenMintB
node -e "import * as anchor from '@coral-xyz/anchor';\
import { PublicKey } from '@solana/web3.js';\
const programId = new PublicKey(process.env.ORIGIN_DEX_PROGRAM_ID);\
const provider = anchor.AnchorProvider.env();\
anchor.setProvider(provider);\
const [registry] = PublicKey.findProgramAddressSync([Buffer.from('registry')], programId);\
const idl = { version: '0.1.0', name: 'origin_dex', instructions: [\
{ name: 'initRegistry', accounts: [ { name: 'registry', isMut: true, isSigner: false }, { name: 'admin', isMut: true, isSigner: true }, { name: 'systemProgram', isMut: false, isSigner: false } ], args: [] },\
{ name: 'createPool', accounts: [ { name: 'registry', isMut: true, isSigner: false }, { name: 'pool', isMut: true, isSigner: false }, { name: 'tokenAMint', isMut: false, isSigner: false }, { name: 'tokenBMint', isMut: false, isSigner: false }, { name: 'admin', isMut: true, isSigner: true }, { name: 'systemProgram', isMut: false, isSigner: false } ], args: [ { name: 'feeBps', type: 'u16' }, { name: 'tokenAPriceCents', type: 'u64' }, { name: 'tokenBPriceCents', type: 'u64' } ] } ] };\
const program = new anchor.Program(idl, programId, provider);\
const regInfo = await provider.connection.getAccountInfo(registry);\
if (!regInfo) { await program.methods.initRegistry().accounts({ registry, admin: provider.wallet.publicKey, systemProgram: anchor.web3.SystemProgram.programId }).rpc(); }\
const regAfter = await provider.connection.getAccountInfo(registry);\
const data = regAfter.data;\
const nextPoolId = Number(data.readBigUInt64LE(8 + 32 + 1));\
const poolSeed = Buffer.alloc(8); poolSeed.writeBigUInt64LE(BigInt(nextPoolId));\
const [pool] = PublicKey.findProgramAddressSync([Buffer.from('pool'), poolSeed], programId);\
const tokenAMint = new PublicKey(process.env.ORIGIN_DEX_TOKEN_A_MINT);\
const tokenBMint = new PublicKey(process.env.ORIGIN_DEX_TOKEN_B_MINT);\
await program.methods.createPool(100, new anchor.BN(100), new anchor.BN(100)).accounts({ registry, pool, tokenAMint, tokenBMint, admin: provider.wallet.publicKey, systemProgram: anchor.web3.SystemProgram.programId }).rpc();\
console.log('Created pool', pool.toBase58());"
```

## Devnet validation checklist
- Solana CLI points to devnet
- Program deployed and program id recorded
- Config PDA initialized
- Registry PDA initialized
- Pool PDA created
- Pool mints have freeze authority
- `clients/ts` reports the config PDA exists
- `clients/py` can read the config PDA
