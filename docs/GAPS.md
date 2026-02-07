# Gaps To "DEX Is Working"

## Missing DEX program logic
- The on-chain program only supports config/registry initialization and creating empty pool records.
- No pool state beyond mints/fees/bin spacing; no swaps, liquidity, or fee accounting.
- Function-based allocation logic is not implemented on-chain.
- One-sided deposits and matching constraints are not implemented.
- LP NFT minting and staking rewards are not implemented.
- Phase 1: no native token rewards; rebate is disabled.
- Redemption guarantees and reserve vaults are not implemented.
- No enforcement for "non-ERC20" guarantee asset beyond manual selection.
- No cross-chain governance mirror logic is implemented.
- Function curves are stored but not executed on-chain (allocation math is off-chain only).
- Position add/close tracks LP NFTs and raw pool totals only.
- Pool liquidity accounting is minimal (raw totals only). No token transfers are enforced on-chain.
- Off-chain bin allocation should be stored in IPFS metadata linked to the LP NFT (not implemented here).

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
