#!/bin/bash

has_container_exited() {
  local container_name="$1"
  if [ -n "$(docker ps -a --filter "name=${container_name}" --filter "status=exited" -q)" ]; then
    return 0  
  else
    return 1
  fi
}

copy_logs() {
  LOGS_DIR="../records/uniswap-proxy-docker-logs"
  mkdir -p $LOGS_DIR
  docker-compose ps -q | xargs -I {} sh -c 'docker logs {} > ../records/uniswap-proxy-docker-logs/$(docker inspect --format="{{.Name}}" {}).log 2>&1'
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

docker-compose up -d solana rome-evm-builder1 proxy rhea

until has_container_exited "rome-evm-builder"; do
  sleep 2
done

airdrop

###############
# Proxy Tests #
###############

echo "Starting Proxy tests..."

# Check balance
if balance_check "http://127.0.0.1:9090" $evm_address 0; then
  echo "Insufficient proxy balance, exiting..."
  exit
fi

docker run --network="local-env_net" --name="uniswap" -e NETWORK='proxy' -e CHAIN_ID='1001' romelabs/uniswap-v2-core:${UNISWAP_V2_TAG:-latest} yarn test | tee ../records/uniswap-proxy.txt

clear_env

echo "Stopping tests..."
