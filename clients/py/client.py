#!/usr/bin/env python3
"""Minimal devnet reader for the Origin DEX config PDA."""

from __future__ import annotations

import base64
import json
import os
import sys
import urllib.request

PROGRAM_ID = os.environ.get(
    "ORIGIN_DEX_PROGRAM_ID", "Orig1nDex111111111111111111111111111111111"
)


def _rpc_call(url: str, method: str, params: list | None = None) -> dict:
    payload = {
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params or [],
    }
    data = json.dumps(payload).encode("utf-8")
    req = urllib.request.Request(url, data=data, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=10) as resp:
        return json.loads(resp.read().decode("utf-8"))


def main() -> int:
    url = os.environ.get("SOLANA_RPC_URL", "https://api.devnet.solana.com")

    # The PDA derivation is done client-side in TS; here we only fetch by address.
    config_pda = os.environ.get("ORIGIN_DEX_CONFIG_PDA")
    if not config_pda:
        print("ORIGIN_DEX_CONFIG_PDA is required for the Python client.")
        return 1

    result = _rpc_call(url, "getAccountInfo", [config_pda, {"encoding": "base64"}])
    value = result.get("result", {}).get("value")
    if value is None:
        print("Config account not found on devnet.")
        return 2

    data_b64 = value.get("data", [None])[0]
    if not data_b64:
        print("Config account has no data.")
        return 3

    raw = base64.b64decode(data_b64)
    print(f"Config account exists. Raw length: {len(raw)} bytes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
