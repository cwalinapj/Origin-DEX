# Local Development

This repo now includes:
- A Python allocation SDK
- A minimal Anchor program (`origin_dex`) with a config PDA
- TypeScript and Python clients that check the config PDA

It is not a full DEX implementation.

## Prereqs
- Python 3.10+
- Node.js 18+
- Rust toolchain
- Solana CLI
- Anchor CLI
- `make`

## Python SDK setup
```bash
cd /Users/root1/scripts/Origin-DEX
python3 -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
pip install -r requirements.txt
```

## Run Python SDK tests
```bash
source .venv/bin/activate
python -m unittest discover -s tests -v
```

## Build whitepaper HTML
```bash
source .venv/bin/activate
make build
open build/index.html
```

## Anchor build (program only)
```bash
anchor build
```

## Anchor tests (initialize config PDA)
```bash
npm install
export ORIGIN_DEX_PROGRAM_ID=ReplaceWithProgramId
anchor test --provider.cluster devnet --provider.wallet ./devnet-wallet.json
```

## Smoke test (RPC connectivity)
```bash
source .venv/bin/activate
python scripts/smoke_devnet_rpc.py
```

## TypeScript client
```bash
cd /Users/root1/scripts/Origin-DEX/clients/ts
npm install
export ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
export ANCHOR_WALLET=/absolute/path/to/devnet-wallet.json
export ORIGIN_DEX_PROGRAM_ID=ReplaceWithProgramId
npm run client
```

## Python client
```bash
cd /Users/root1/scripts/Origin-DEX/clients/py
export SOLANA_RPC_URL=https://api.devnet.solana.com
export ORIGIN_DEX_PROGRAM_ID=ReplaceWithProgramId
export ORIGIN_DEX_CONFIG_PDA=ReplaceWithConfigPda
python client.py
```
