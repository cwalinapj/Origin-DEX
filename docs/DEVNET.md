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
- `token_a_kind` / `token_b_kind`:
  - `1` ERC20 proxy (mint must be frozen)
  - `2` Fiat/Gold proxy (mint must be **unfrozen**)
  - `3` Wrapped SOL (no freeze requirement)
  - `4` USDC (mint must be frozen)
  - `5` EUR token (mint must be frozen)
  - `6` Commodity proxy (mint must be frozen)
  - `7` Native token (mint must be frozen)
- `guarantee_policy`:
  - `0` fixed mint (set `guarantee_mint`, `allowed_assets_mask = 0`)
  - `1` user choice (set `allowed_assets_mask`, `guarantee_mint = default`)
- `allowed_assets_mask` bits for user choice:
  - `1` WSOL
  - `2` USDC
  - `4` Native token
  - `8` EUR token
  - `16` Fiat/Gold proxy
  - `32` Commodity proxy
  - Note: ERC20 proxies are intentionally excluded from user-choice guarantees.

Position allocation parameters (stored on-chain per LP position):
- `min_price_cents` / `max_price_cents`:
  - Absolute price bounds for the position range.
  - Bin sampling uses **one bin per price interval** between min/max.
- `left_function_type` / `right_function_type`:
  - `1` linear: `f(x)=m(x−x0)+y0`
  - `2` log: `g(x)=A⋅log_B(C(−x+h))+k`
- `left_params` / `right_params`: fixed-point integers scaled by `1e6`
  - Linear params: `[m, x0, y0, unused, unused]`
  - Log params: `[A, B, C, h, k]`
- `amount_a` / `amount_b`: raw token amounts for the position
  - One-sided deposits are only allowed when the **other side remains >= 50% of total value** (based on pool price cents).

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
{ name: 'createPool', accounts: [ { name: 'registry', isMut: true, isSigner: false }, { name: 'pool', isMut: true, isSigner: false }, { name: 'tokenAMint', isMut: false, isSigner: false }, { name: 'tokenBMint', isMut: false, isSigner: false }, { name: 'admin', isMut: true, isSigner: true }, { name: 'systemProgram', isMut: false, isSigner: false } ], args: [ { name: 'feeBps', type: 'u16' }, { name: 'tokenAPriceCents', type: 'u64' }, { name: 'tokenBPriceCents', type: 'u64' }, { name: 'tokenAKind', type: 'u8' }, { name: 'tokenBKind', type: 'u8' }, { name: 'guaranteePolicy', type: 'u8' }, { name: 'allowedAssetsMask', type: 'u16' }, { name: 'guaranteeMint', type: 'publicKey' } ] } ] };\
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
const guaranteePolicy = 1; /* user choice */\
const allowedAssetsMask = 3; /* WSOL + USDC */\
await program.methods.createPool(100, new anchor.BN(100), new anchor.BN(100), 4, 3, guaranteePolicy, allowedAssetsMask, PublicKey.default).accounts({ registry, pool, tokenAMint, tokenBMint, admin: provider.wallet.publicKey, systemProgram: anchor.web3.SystemProgram.programId }).rpc();\
console.log('Created pool', pool.toBase58());"
```

## Devnet validation checklist
- Solana CLI points to devnet
- Program deployed and program id recorded
- Config PDA initialized
- Registry PDA initialized
- Pool PDA created
- Pool mints have freeze authority
- LP position can be minted and staked
- `clients/ts` reports the config PDA exists
- `clients/py` can read the config PDA

Note: Phase 1 has **no native token rewards**. Staking is a custody/eligibility primitive only.
