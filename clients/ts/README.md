# TypeScript Client

## Setup
```bash
cd /Users/root1/scripts/Origin-DEX/clients/ts
npm install
```

## Run
```bash
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
export ANCHOR_WALLET=/absolute/path/to/devnet-wallet.json
export ORIGIN_DEX_PROGRAM_ID=ReplaceWithProgramId
npm run client
```

Notes:
- This client only derives the config PDA and checks if it exists.
- Initialize the program using Anchor after deployment.
