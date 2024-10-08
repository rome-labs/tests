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
  LOGS_DIR="../records/openzeppelin-geth-docker-logs"
  mkdir -p $LOGS_DIR
  docker-compose ps -q | xargs -I {} sh -c 'docker logs {} > ../records/openzeppelin-geth-docker-logs/$(docker inspect --format="{{.Name}}" {}).log 2>&1'
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

docker-compose up -d solana rome-evm-builder1 proxy geth rhea

until has_container_exited "rome-evm-builder"; do
  sleep 2
done

airdrop

#################
# Op-Geth Tests #
#################

echo "Starting Op-Geth tests..."

docker-compose up -d solana rome-evm-builder1 proxy geth rhea

# Check balance
if balance_check "http://127.0.0.1:8545" $evm_address 0; then
  echo "Insufficient op_geth balance, exiting..."
  exit
fi

docker run --network="local-env_net" --name="openzeppelin" romelabs/openzeppelin-contracts:${OPENZEPPLIN_TAG:-latest} -env NETWORK_NAME='op_geth' | tee ../records/zeppelin-op-geth.txt

clear_env

echo "Stopping tests..."
