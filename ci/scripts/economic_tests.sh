#!/bin/bash
source ./ci/scripts/functions.sh

set_default_env() {
  SOL_RPC_URL="http://localhost:8899"
  ETH_RPC_URL="http://localhost:9090"
}


get_solana_balance() {
  set_default_env
  SOLANA=$(get_container_id solana)
  SOL_BALANCE=$(docker exec $SOLANA solana balance $1 -u $SOL_RPC_URL | awk '{print $1}')
  LAMPORTS=$(printf "%.0f" "$(echo "$SOL_BALANCE * 1000000000" | bc)")
  echo "$LAMPORTS"
}

get_eth_balance() {
  set_default_env
  RESPONSE=$(curl -s -X POST --data '{"jsonrpc":"2.0","method":"eth_getBalance","params":["'"$1"'", "latest"],"id":1}' -H "Content-Type: application/json" "$ETH_RPC_URL")
  # Extract the result field (hex string), convert to decimal
  BAL_HEX=$(echo "$RESPONSE" | jq -r .result)
  # Convert hex to decimal using `bc`
  BAL_DEC=$(echo "$BAL_HEX" | awk '{print strtonum($1)}')
  # Convert RSOL to lamports
  LAMPORTS=$(printf "%.0f" "$(echo "scale=18; $BAL_DEC / 1000000000" | bc)")
  echo "$LAMPORTS"
}
