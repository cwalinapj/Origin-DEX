# Local Development

This repo contains a Python SDK (allocation math) and a whitepaper. It does not include an on-chain Solana program.

## Prereqs
- Python 3.10+
- `make`

## Setup
```bash
cd /Users/root1/scripts/Origin-DEX
python3 -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
pip install -r requirements.txt
```

## Run tests
```bash
source .venv/bin/activate
python -m unittest -v
```

## Build whitepaper HTML
```bash
source .venv/bin/activate
make build
open build/index.html
```

## Smoke test (RPC connectivity)
```bash
source .venv/bin/activate
python scripts/smoke_devnet_rpc.py
```

Notes:
- `make build` requires `markdown` from `requirements.txt`.
- The smoke test uses only Python standard library and the public Solana devnet RPC.
