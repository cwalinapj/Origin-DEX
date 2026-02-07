# Python Client

## Run
```bash
cd /Users/root1/scripts/Origin-DEX/clients/py
export SOLANA_RPC_URL=https://api.devnet.solana.com
export ORIGIN_DEX_PROGRAM_ID=ReplaceWithProgramId
export ORIGIN_DEX_CONFIG_PDA=ReplaceWithConfigPda
python client.py
```

Notes:
- This client only reads the config PDA and does not sign transactions.
- Use the TypeScript client or Anchor to derive the PDA address.
