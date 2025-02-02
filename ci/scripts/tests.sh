#!/bin/bash

copy_logs() {
  LOGS_DIR="../records/tests-docker-logs"
  mkdir -p $LOGS_DIR
  docker-compose ps -aq | xargs -I {} sh -c 'docker logs {} > ../records/tests-docker-logs/$(docker inspect --format="{{.Name}}" {}).log 2>&1'
}

clear_env() {
  copy_logs
  docker-compose down
  docker container prune -f
}

airdrop() {
  docker cp ./keys/rhea-sender.json solana:./
  docker cp ./keys/proxy-sender.json solana:./
  docker cp ./keys/rollup-owner-keypair.json solana:./
  docker cp ./keys/test-account-keypair.json solana:./
  docker cp ./keys/upgrade-authority-keypair.json solana:./

  docker exec solana solana -u http://localhost:8899 airdrop 10000 ./proxy-sender.json
  docker exec solana solana -u http://localhost:8899 airdrop 10000 ./rhea-sender.json
  docker exec solana solana -u http://localhost:8899 airdrop 10000 ./rollup-owner-keypair.json
  docker exec solana solana -u http://localhost:8899 airdrop 10000 ./test-account-keypair.json
  docker exec solana solana -u http://localhost:8899 airdrop 10000 ./upgrade-authority-keypair.json
}

mkdir -p records
touch ./records/tests-evm.txt
cd ./ci
docker-compose up -d solana
sleep 5
airdrop

###############
# Tests #
###############

echo "Starting tests..."

docker run --network="ci_net" -e TEST_NAME=evm --name="tests" romelabs/tests:${TESTS_TAG:-latest} | tee ../records/tests-evm.txt

clear_env

echo "Stopping tests..."
