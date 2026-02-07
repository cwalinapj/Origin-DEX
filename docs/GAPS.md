# Gaps To "DEX Is Working"

## Missing DEX program logic
- The on-chain program only supports config/registry initialization and creating empty pool records.
- No pool state beyond mints/fees/bin spacing; no swaps, liquidity, or fee accounting.
- Function-based allocation logic is not implemented on-chain.
- One-sided deposits and matching constraints are not implemented.
- LP NFT minting and staking rewards are not implemented.
- House-fee rebate in native token is not implemented.

## Missing IDL and client bindings
- No generated IDL or typed client.
- Clients are minimal and only read the config PDA.

## Missing devnet operational flows
- No pool creation or initialization instructions.
- No liquidity seeding scripts.
- No swap execution scripts or examples.
- No account/seed layout documentation for DEX state.

## Missing tests
- Only unit tests for allocation math exist.
- No Anchor tests or devnet integration tests.

## Missing scripts referenced by docs
- README references `scripts/setup_devnet_wallet_macos_v2.sh`. A placeholder exists but is not implemented.
