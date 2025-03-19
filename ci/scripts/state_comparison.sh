#!/bin/bash

copy_logs() {
  LOGS_DIR="../records/tests-docker-logs"
  mkdir -p $LOGS_DIR
  docker-compose ps -aq | xargs -I {} sh -c 'docker logs {} > ../records/tests-docker-logs/$(docker inspect --format="{{.Name}}" {}).log 2>&1'
}

has_container_exited() {
  local container_name="$1"
  if [ -n "$(docker ps -a --filter "name=${container_name}" --filter "status=exited" -q)" ]; then
    return 0  
  else
    return 1
  fi
}

clear_env() {
  copy_logs
  docker-compose down > /dev/null
  docker container prune -f > /dev/null
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

  echo "Reg_rollup..."
  docker-compose up -d reg_rollup --no-recreate
  until has_container_exited "reg_rollup"; do
    sleep 3
  done


  echo "deposit..."
  docker-compose up -d deposit --no-recreate
  until has_container_exited "deposit"; do
    sleep 3
  done
}

mkdir -p records
touch ./records/state_comparison.txt
cd ./ci

echo "Docker-compose..."
docker-compose up --quiet-pull -d solana proxy hercules rhea geth faucet postgres > /dev/null

sleep 10
echo "Airdrop..."
airdrop

###############
# Tests #
###############

echo "Starting tests..."

docker run --network="ci_net" -e PROXY_URL=$PROXY_URL -e GETH_URL=$GETH_URL -e TEST_NAME=state_comparison -e EXTENDED_LOGS=$EXTENDED_LOGS --name="tests" romelabs/tests:${TESTS_TAG:-latest} | tee ../records/state_comparison.txt

if ! cat ../records/state_comparison.txt | grep '; 0 failed;'; then
  echo "Tests failed. Exiting with error."
  clear_env
  exit 1
else
  echo "Tests passed. Stopping tests..."
  clear_env
fi
