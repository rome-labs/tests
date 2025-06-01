#!/bin/bash
source ./ci/scripts/functions.sh

filename=$(basename -- "$0") 
TEST_NAME="${filename%.*}"
echo "Test name: $TEST_NAME"
$(create_log_file $TEST_NAME)

evm_address="0x768b73EE6CA9e0A1bc32868CA65dB89E44696DD8"

cd ./ci
docker-compose up -d solana proxy geth rhea hercules faucet > /dev/null
sleep 15

airdrop
airdrop_to_wallets
regrollup
deposit

###############
# Load Tests #
###############

# cd ./xk6

docker-compose -f ./xk6/$TEST_NAME.yml up -d
echo "Started all load test container..."

echo "Running load tests inside k6..."
docker exec xk6 bash -c "./k6 run --out json=records/results.json scenarios/proxy.js && cp summary.html records/report/ && cp summary.txt records/report/" | tee ../records/$TEST_NAME.txt

echo "Finished load tests with k6"
docker ps -a
docker images

clear_env
