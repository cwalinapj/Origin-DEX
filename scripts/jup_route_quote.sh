#!/usr/bin/env bash
set -euo pipefail

# Jupiter quote helper (off-chain). Requires network access.
# Usage:
#   INPUT_MINT=So11111111111111111111111111111111111111112 \
#   OUTPUT_MINT=4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU \
#   INPUT_AMOUNT=1000000000 \
#   bash scripts/jup_route_quote.sh

: "${INPUT_MINT:?INPUT_MINT required}"
: "${OUTPUT_MINT:?OUTPUT_MINT required}"
: "${INPUT_AMOUNT:?INPUT_AMOUNT required}"

curl -s "https://quote-api.jup.ag/v6/quote?inputMint=${INPUT_MINT}&outputMint=${OUTPUT_MINT}&amount=${INPUT_AMOUNT}" | jq '.'
