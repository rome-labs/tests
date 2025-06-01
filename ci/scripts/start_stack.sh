#!/bin/bash
source ./ci/scripts/functions.sh

filename=$(basename -- "$0") 
TEST_NAME="${filename%.*}"
echo "Test name: $TEST_NAME"
$(create_log_file $TEST_NAME)

evm_address="0x768b73EE6CA9e0A1bc32868CA65dB89E44696DD8"

cd ./ci
docker-compose up --quiet-pull -d solana proxy geth rhea hercules faucet > /dev/null

# Wait while geth started
sleep 15

airdrop
regrollup
deposit
$(get_logs_from faucet) 
airdrop_to_wallets

# Check balance
if balance_check "http://127.0.0.1:9090" $evm_address 0; then
  echo "Insufficient proxy balance, exiting..."
  exit 1
fi
