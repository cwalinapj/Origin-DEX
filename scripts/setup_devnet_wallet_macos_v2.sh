#!/usr/bin/env bash
set -euo pipefail

cat <<'MSG'
TODO: This script is referenced by the whitepaper but is not implemented in this repo.

Expected behavior (inferred from README):
- Create or select a Solana keypair
- Airdrop SOL on devnet
- Mint or obtain Circle devnet USDC

Until this is implemented, set up a wallet manually and fund it using:
  solana-keygen new --outfile ./devnet-wallet.json
  solana config set --keypair ./devnet-wallet.json
  solana config set --url https://api.devnet.solana.com
  solana airdrop 2

For devnet USDC, use a Circle devnet faucet or your preferred mint flow.
MSG

exit 1
