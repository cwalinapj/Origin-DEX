#!/usr/bin/env python3
"""Minimal Solana devnet RPC smoke test (no external deps)."""

from __future__ import annotations

import json
import os
import sys
import urllib.request


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

    try:
        health = _rpc_call(url, "getHealth")
        version = _rpc_call(url, "getVersion")
    except Exception as exc:  # pragma: no cover - diagnostic path
        print(f"RPC call failed: {exc}")
        return 1

    if health.get("result") != "ok":
        print(f"Unexpected health response: {health}")
        return 1

    if "result" not in version or "solana-core" not in version["result"]:
        print(f"Unexpected version response: {version}")
        return 1

    print("RPC health: ok")
    print(f"Solana version: {version['result'].get('solana-core')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
