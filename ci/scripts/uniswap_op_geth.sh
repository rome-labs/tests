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
  LOGS_DIR="../records/uniswap-geth-docker-logs"
  mkdir -p $LOGS_DIR
  docker-compose ps -aq | xargs -I {} sh -c 'docker logs {} > ../records/uniswap-geth-docker-logs/$(docker inspect --format="{{.Name}}" {}).log 2>&1'
}

clear_env() {
  copy_logs
  docker-compose down > /dev/null
}

airdrop() {
  docker cp ./keys/rhea-sender.json solana:./
  docker cp ./keys/proxy-sender.json solana:./
  docker cp ./keys/test-account-keypair.json solana:./
  docker cp ./keys/upgrade-authority-keypair.json solana:./

  docker exec solana solana -u http://localhost:8899 airdrop 10000 ./proxy-sender.json
  docker exec solana solana -u http://localhost:8899 airdrop 10000 ./rhea-sender.json
  docker exec solana solana -u http://localhost:8899 airdrop 10000000 ./test-account-keypair.json
  docker exec solana solana -u http://localhost:8899 airdrop 10000 ./upgrade-authority-keypair.json

  echo "regrollup..."
  docker-compose up --quiet-pull -d reg_rollup --no-recreate > /dev/null
  until has_container_exited "reg_rollup"; do
    sleep 2
  done

  echo "deposit ..."
  docker-compose up --quiet-pull -d deposit --no-recreate > /dev/null
  until has_container_exited "deposit"; do
    sleep 2
  done

  docker logs faucet

  echo "Starting Airdrop..."
  curl -m 5 --location 'http://localhost:3000/airdrop' --header 'Content-Type: application/json' --data '{"recipientAddr": "0xa3349dE31ECd7fd9413e1256b6472a68c920D186", "amount": "100.0"}'
  curl -m 5 --location 'http://localhost:3000/airdrop' --header 'Content-Type: application/json' --data '{"recipientAddr": "0x6970d087e7e78A13Ea562296edb05f4BB64D5c2E", "amount": "100.0"}'
  curl -m 5 --location 'http://localhost:3000/airdrop' --header 'Content-Type: application/json' --data '{"recipientAddr": "0xaA4d6f4FF831181A2bBfD4d62260DabDeA964fF1", "amount": "100.0"}'
  echo "Finished Airdrop..."

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
touch ./records/uniswap_op_geth.txt
cd ./ci
docker ps

echo "Starting docker-compose..."
docker-compose up --quiet-pull -d solana proxy geth rhea hercules faucet postgres > /dev/null

# Wait while geth started
sleep 15

airdrop

# source ./ci/scripts/start_stack.sh


#################
# Op Geth Tests #
#################

echo "Starting Op Geth tests..."


# Check balance
if balance_check "http://127.0.0.1:9090" $evm_address 0; then
  echo "Insufficient proxy balance, exiting..."
  exit 1
fi

# Check if an argument was provided
if [ -n "$1" ]; then
  SUIT=$1
  docker run --network="ci_net" --name="uniswap" -e NETWORK='op-geth' -e CHAIN_ID='1001' romelabs/uniswap-v2-core:${UNISWAP_V2_TAG:-latest} yarn test --grep "$SUIT" | tee ../records/uniswap_op_geth$SUIT.txt
else
  docker run --network="ci_net" --name="uniswap" -e NETWORK='op-geth' -e CHAIN_ID='1001' romelabs/uniswap-v2-core:${UNISWAP_V2_TAG:-latest} yarn test | tee ../records/uniswap_op_geth.txt
fi


if cat ../records/uniswap_op_geth$SUIT.txt | grep -E '[1-9][0-9]? failing'; then
  echo "Tests failed. Exiting with error."
  clear_env
  exit 1
else
  echo "Tests passed. Stopping tests..."
  clear_env
fi
