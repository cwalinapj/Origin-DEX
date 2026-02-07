# Devnet Readiness

This repo is not a deployable Solana program. It only includes a Python allocation SDK and a whitepaper. The steps below validate devnet connectivity and document expected setup for any future on-chain program.

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

## Devnet RPC smoke test
```bash
python scripts/smoke_devnet_rpc.py
```

## Program deployment
No on-chain program, `Anchor.toml`, `programs/`, or Rust source exists in this repo. There is nothing to build or deploy to devnet from this codebase.

## Validation checklist
- Solana CLI points to devnet
- A funded devnet wallet exists
- `python scripts/smoke_devnet_rpc.py` returns `RPC health: ok`
