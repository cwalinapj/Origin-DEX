# Gaps To "DEX Is Working"

## Missing on-chain program
- No `Anchor.toml`, `programs/`, `Cargo.toml`, or Rust source present.
- No IDL, migrations, or deployment tooling.
- No program ID to verify on devnet.

## Missing scripts referenced by docs
- README references `scripts/setup_devnet_wallet_macos_v2.sh`. A placeholder exists but is not implemented.

## Missing devnet operational flows
- No pool creation or initialization instructions.
- No liquidity seeding scripts.
- No swap execution scripts or examples.
- No account/seed layout documentation.

## Missing integration boundary
- The SDK only performs allocation math and does not connect to Solana.
- No client bindings or transaction builders for creating positions, pools, or swaps.

## Missing tests
- Only unit tests for allocation math exist.
- No integration tests against devnet.
