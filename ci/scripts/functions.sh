#!/bin/bash

get_container_id() {
  docker-compose ps -q "$1"
}

has_container_exited() {
  local container_name="$1"
  if [ -n "$(docker ps -a --filter "name=${container_name}" --filter "status=exited" -q)" ]; then
    return 0
  else
    return 1
  fi
}

copy_logs() {
  LOGS_DIR="../records/docker-logs"
  mkdir -p $LOGS_DIR
  docker-compose ps -aq | xargs -I {} sh -c 'docker logs {} > ../records/docker-logs/$(docker inspect --format="{{.Name}}" {}).log 2>&1'
}

clear_env() {
  if [ "$GITHUB_ACTIONS" = "true" ]; then
    echo "Running in GitHub Actions CI"
    echo "clearing will be done by actions workflow..."
  else
    echo "Running locally"
    copy_logs
    docker-compose down > /dev/null
    docker container prune -f
  fi
}

get_logs_from() {
  CONTAINER=$(get_container_id $1)
  echo "container_id: $CONTAINER"
  docker logs $CONTAINER
}

get_key_of() {
  SOLANA=$(get_container_id solana)
  docker exec $SOLANA solana -u http://localhost:8899 address -k $1
}

airdrop() {
  SOLANA=$(get_container_id solana)
  echo "container_id: $SOLANA"
  
  docker cp ./keys/rhea-sender.json $SOLANA:./
  docker cp ./keys/proxy-sender.json $SOLANA:./
  docker cp ./keys/test-account-keypair.json $SOLANA:./
  docker cp ./keys/upgrade-authority-keypair.json $SOLANA:./
  docker cp ./keys/empty-keypair.json $SOLANA:./

  docker exec $SOLANA solana -u http://localhost:8899 airdrop 1 ./empty-keypair.json
  docker exec $SOLANA solana -u http://localhost:8899 airdrop 10000 ./proxy-sender.json
  docker exec $SOLANA solana -u http://localhost:8899 airdrop 10000 ./rhea-sender.json
  docker exec $SOLANA solana -u http://localhost:8899 airdrop 410000000 ./test-account-keypair.json
  docker exec $SOLANA solana -u http://localhost:8899 airdrop 10000 ./upgrade-authority-keypair.json
}

airdrop_to_wallets() {
  local max_retries=3
  local attempt=1
  local success=0

  while [ $attempt -le $max_retries ]; do
    echo "Airdrop to wallets attempt $attempt..."
    if [ $attempt -gt 1 ]; then
      local delay=$((RANDOM % 3 + 1))
      echo "Sleeping for $delay seconds before retry..."
      sleep $delay
    fi

    # fail if any fails
    local all_ok=0
    for addr in \
      "0xa3349dE31ECd7fd9413e1256b6472a68c920D186" \
      "0x6970d087e7e78A13Ea562296edb05f4BB64D5c2E" \
      "0xaA4d6f4FF831181A2bBfD4d62260DabDeA964fF1"
    do
      response=$(curl -s -m 5 --location 'http://localhost:3000/airdrop' \
        --header 'Content-Type: application/json' \
        --data "{\"recipientAddr\": \"$addr\", \"amount\": \"100.0\"}")

      # Check for "success":true in the response
      if ! echo "$response" | grep -q '"success":true'; then
        echo "Airdrop failed for $addr: $response"
        all_ok=1
      fi
    done
    
    if [ $all_ok -eq 0 ]; then
      echo "Finished Airdrop..."
      success=1
      break
    else
      echo "Airdrop to wallets failed on attempt $attempt."
    fi

    attempt=$((attempt + 1))
  done

  if [ $success -ne 1 ]; then
    echo "Airdrop to wallets failed after $max_retries attempts."
    return 1
  fi
}

regrollup() {
  echo "regrollup..."
  docker-compose up --quiet-pull -d reg_rollup --no-recreate > /dev/null
  until has_container_exited "reg_rollup"; do
    sleep 2
  done
}

deposit() {
  echo "deposit ..."
  docker-compose up --quiet-pull -d deposit --no-recreate > /dev/null
  until has_container_exited "deposit"; do
    sleep 2
  done
}

get_docker_info() {
  docker ps -a
  lsof -i :9090
  docker network ls
  docker ps --filter "name=proxy" --filter "status=running" -q
}

create_log_file() {
  mkdir -p records
  touch ./records/"$1".txt
}

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
# solana proxy geth rhea hercules faucet postgres
mint_spl() {
  SOLANA=$(get_container_id solana)
  docker exec $SOLANA ls
  docker exec $SOLANA solana -u http://localhost:8899 airdrop 100 ./mint-authority.json
  MINT_ADDRESS=$(docker exec $SOLANA spl-token create-token --mint-authority ./mint-authority.json -u http://localhost:8899 ./mint-keypair.json | grep '^Address:' | awk '{print $2}')
  echo "Mint Address: $MINT_ADDRESS"
}

copy_solana_keys() {
  SOLANA=$(get_container_id solana)
  docker cp ./keys/mint-keypair.json $SOLANA:./
  docker cp ./keys/mint-authority.json $SOLANA:./
}

airdrop_oz() {
  SOLANA=$(get_container_id solana)
  docker cp ./keys/id1.json $SOLANA:./

  docker exec $SOLANA solana -u http://localhost:8899 airdrop 10000
  docker exec $SOLANA solana -u http://localhost:8899 airdrop 10000 ./id1.json
}
