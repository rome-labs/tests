#!/bin/bash
source ./ci/scripts/functions.sh

filename=$(basename -- "$0") 
TEST_NAME="${filename%.*}"
echo "Test name: $TEST_NAME"
$(create_log_file $TEST_NAME)

cd ./ci
docker-compose up --quiet-pull -d solana
sleep 5
airdrop

###############
# Tests #
###############


echo "Starting tests..."

docker run --network="ci_net" \
  -e PROXY_URL=$PROXY_URL \
  -e GETH_URL=$GETH_URL \
  -e TEST_NAME=$TEST_NAME \
  -e TEST_ACCOUNT=$TEST_ACCOUNT \
  --name="tests" \
  romelabs/tests:${TESTS_TAG:-latest} | tee ../records/$TEST_NAME.txt

if ! cat ../records/$TEST_NAME.txt | grep '; 0 failed;'; then
  echo "Tests failed. Exiting with error."
  clear_env
  exit 1
else
  echo "Tests passed. Stopping tests..."
  clear_env
fi
