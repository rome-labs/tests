#!/bin/bash

has_container_exited() {
  local container_name="$1"
  if [ -n "$(docker ps -a --filter "name=${container_name}" --filter "status=exited" -q)" ]; then
    return 0  
  else
    return 1
  fi
}

export ROME_EVM_BUILDER_TAG = "v0.1.0"
export GETH_TAG = "v0.1.0"
export RHEA_TAG = "v0.1.0"
export PROXY_TAG = "v0.1.0"

clear_env() {
  docker-compose down 
  docker stop uniswap openzeppelin storage rome-tests
  docker rm uniswap openzeppelin storage rome-tests
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

docker-compose up -d

until has_container_exited "rome-evm-builder"; do
  sleep 2
done

airdrop

# Check balance
if balance_check "http://127.0.0.1:9090" $evm_address 0; then
  echo "Insufficient proxy balance, exiting..."
  exit
fi

###############
# Rome Tests #
###############

echo "Starting rome tests..."

# Check balance
if balance_check "http://127.0.0.1:9090" $evm_address 0; then
  echo "Insufficient proxy balance, exiting..."
  exit
fi

# Run pair deployments 
docker run --network="local-env_net" -e NETWORK='proxy' -e CHAIN_ID='1001' romeprotocol/uniswap-v2-core:latest yarn deploy:uniswapv2crossrollup
docker run --network="local-env_net" -e NETWORK='proxy2' -e CHAIN_ID='1002' romeprotocol/uniswap-v2-core:latest yarn deploy:uniswapv2crossrollup

docker run --network="local-env_net" --name="rome-tests" -e CROSS_ROLLUP_TESTS=true romeprotocol/tests:latest | tee ../records/rome_tests.txt

clear_env

echo "Stopping tests..."
