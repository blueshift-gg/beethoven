#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"

RPC_URL="${RPC_URL:-https://mainnet.helius-rpc.com/?api-key=f08679a5-b3bb-44e6-840a-a330150160d7}"

fetch() {
  local pubkey="$1"
  local out="$2"
  solana account "$pubkey" -u "$RPC_URL" --output json -o "$out"
}

fetch 2zmg7259ahZkrSn6M3PEM7eFvEfBU8obgfVHT3AL9Qwu fixtures/swap/pancake/amm_config.json
fetch 4QU2NpRaqmKMvPSwVKQDeW4V6JFEKJdkzbzdauumD9qN fixtures/swap/pancake/sol_usdc_pool_state.json
fetch 2h4rB9TSGFehKrvKzrMM4RrWqBefGV1STdpekc3cTyBy fixtures/swap/pancake/sol_usdc_observation_state.json
fetch 8JL1ZnyMvd48AqF9YTCE4UnEV8dDydU2cYQSa6bzykYP fixtures/swap/pancake/sol_vault.json
fetch FVDSv2aymcXu5ubgFePqPmp24nrKiwpUFc21b2Kqh6gC fixtures/swap/pancake/usdc_vault.json
fetch AP4Jafrw8jzASGrgZHEiNvYxhdapUYPAqxFiMsKLyvXu fixtures/swap/pancake/tick_array_bitmap_extension.json
fetch Gy4zp3Kz5N5UKhcMyNE1ymgrncTcDd6b3FD9sw9EH1k5 fixtures/swap/pancake/tick_array_state_1.json
fetch 3Z151sJtAMP2KGsj9oYF5UF9cnKSWaGwGaoAgjGt8hUo fixtures/swap/pancake/tick_array_state_2.json
fetch EHekGmRXkTVjztDwSCnfaxi6D3QpZLXnj31XCTdat9Z3 fixtures/swap/pancake/tick_array_state_3.json