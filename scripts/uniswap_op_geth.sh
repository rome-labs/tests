#!/bin/bash

copy_logs() {
  LOGS_DIR="../records/uniswap-geth-docker-logs"
  mkdir -p $LOGS_DIR
  docker-compose ps -q | xargs -I {} sh -c 'docker logs {} > ../records/uniswap-geth-docker-logs/$(docker inspect --format="{{.Name}}" {}).log 2>&1'
}

clear_env() {
  copy_logs
  docker-compose down
}

airdrop() {
  docker cp rhea-sender.json solana:./
  docker cp proxy-sender.json solana:./
  docker cp ../ci/rome-owner-keypair.json solana:./
  docker cp ../ci/test-account-keypair.json solana:./
  docker cp ../ci/rollup-tx-payer.json solana:./

  docker exec solana solana -u http://localhost:8899 airdrop 10000 ./proxy-sender.json
  docker exec solana solana -u http://localhost:8899 airdrop 10000 ./rhea-sender.json
  docker exec solana solana -u http://localhost:8899 airdrop 10000 ./rome-owner-keypair.json
  docker exec solana solana -u http://localhost:8899 airdrop 10000 ./test-account-keypair.json
  docker exec solana solana -u http://localhost:8899 airdrop 10000 ./rollup-tx-payer.json

  curl --location 'http://localhost:3000/airdrop' --header 'Content-Type: application/json' --data '{"recipientAddr": "0xa3349dE31ECd7fd9413e1256b6472a68c920D186", "amount": "100.0"}'
  curl --location 'http://localhost:3000/airdrop' --header 'Content-Type: application/json' --data '{"recipientAddr": "0x6970d087e7e78A13Ea562296edb05f4BB64D5c2E", "amount": "100.0"}'
  curl --location 'http://localhost:3000/airdrop' --header 'Content-Type: application/json' --data '{"recipientAddr": "0xaA4d6f4FF831181A2bBfD4d62260DabDeA964fF1", "amount": "100.0"}'
}

evm_address="0x768b73EE6CA9e0A1bc32868CA65dB89E44696DD8"

balance_check() {
  local rpc_url="$1"
  local address="$2"

  response=$(curl -s $rpc_url \
    -X POST \
    -H "Content-Type: application/json" \
    --data '{"method":"eth_getBalance","params":["'$address'", "latest"],"id":1,"jsonrpc":"2.0"}')

  balance=$(echo "$response" | sed -n 's/.*"result":"\([^"]*\)".*/\1/p')
  if [ -z "$balance" ] || [ "$balance" == "0x0" ]; then
    clear_env
    return 0
  else
    return 1
  fi
}


mkdir -p records
cd ./local-env

docker-compose up -d solana rome-evm-builder1 proxy geth rhea

# Wait while geth started
sleep 15

airdrop

#################
# Op Geth Tests #
#################

echo "Starting Op Geth tests..."

# Check balance
if balance_check "http://127.0.0.1:9090" $evm_address 0; then
  echo "Insufficient proxy balance, exiting..."
  exit
fi

docker run --network="local-env_net" --name="uniswap" -e NETWORK='op-geth' -e CHAIN_ID='1001' romelabs/uniswap-v2-core:${UNISWAP_V2_TAG:-latest} yarn test | tee ../records/uniswap-op-geth.txt

clear_env

echo "Stopping tests..."
