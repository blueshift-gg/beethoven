#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

RPC_URL="${RPC_URL:-https://mainnet.helius-rpc.com/?api-key=f08679a5-b3bb-44e6-840a-a330150160d7}"

fetch() {
  local pubkey="$1"
  local out="$2"
  solana account "$pubkey" -u "$RPC_URL" --output json -o "$out"
}

fetch 9PWf4kEwa3E4WCMnPp4SQoUGWNaA8Zn427g33n6jcmMb fixtures/deposit/carrot-boost/clend_group.json
fetch HwEujdhizP5gpHC63a6xF9qWjo2NvKvdTJdNEhHY9hhK fixtures/deposit/carrot-boost/clend_account.json
fetch 4a74Z8rY6JuuTUeVv7i8kB7LQRANb72jMtweFTUoQM81 fixtures/deposit/carrot-boost/usdc_bank.json
fetch 4ZU6vJULZNxP9BQzRgc5UFtzrSJhs77An9iA6W9ceUEq fixtures/deposit/carrot-boost/usdc_vault.json
fetch Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX fixtures/deposit/carrot-boost/usdc_price_update_v2.json