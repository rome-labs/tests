#!/bin/bash
source ./ci/scripts/functions.sh

filename=$(basename -- "$0") 
TEST_NAME="${filename%.*}"
echo "Test name: $TEST_NAME"
$(create_log_file $TEST_NAME)


cd ./ci
echo "Docker-compose..."
docker-compose up --quiet-pull -d solana proxy hercules rhea geth faucet postgres > /dev/null

sleep 5
echo "Airdrop..."
airdrop
copy_solana_keys
regrollup
deposit
mint_spl

###############
# Tests #
###############

echo "Starting tests..."

docker run --network="ci_net" -e MINT_ADDRESS=$MINT_ADDRESS -e PROXY_URL=$PROXY_URL -e GETH_URL=$GETH_URL -e TEST_NAME=$TEST_NAME -e EXTENDED_LOGS=$EXTENDED_LOGS --name="tests" romelabs/tests:${TESTS_TAG:-latest} | tee ../records/$TEST_NAME.txt

if ! cat ../records/$TEST_NAME.txt | grep '; 0 failed;'; then
  echo "Tests failed. Exiting with error."
  clear_env
  exit 1
else
  echo "Tests passed. Stopping tests..."
  clear_env
fi
