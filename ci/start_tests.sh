#!/bin/bash

sleep 1
SOLANA_RPC_URL=${SOLANA_RPC:-http://solana:8899}
/opt/bin/solana config set --url $SOLANA_RPC_URL
/opt/bin/solana-keygen new --no-bip39-passphrase --silent
if [ -z "$SOLANA_RPC" ]; then 
  /opt/bin/solana airdrop -k /opt/ci/keys/test-account-keypair.json 100
  /opt/bin/solana airdrop -k /opt/ci/keys/upgrade-authority-keypair.json 100
fi

if [ -z "$TEST_NAME" ]; then
  echo "TEST_NAME is not set"
  exit 1
fi

if [ -n "$CI_ENV" ]; then
  echo "Running in real testnet"
  CI_ARG="--skip iter_rw_atomic_ro"
fi

echo "PRINTENV SH:"
printenv EXTENDED_LOGS
cd /opt/bin
if [ "$TEST_NAME" == "state_comparison" ] || [ "$TEST_NAME" == "evm" ]; then
  echo "1. Running $TEST_NAME $EXTENDED_LOGS $CI_ARG --test-threads=1 "
  ./$TEST_NAME $EXTENDED_LOGS $CI_ARG --test-threads=1 
else
  echo "2. Running $TEST_NAME $CI_ARG --test-threads=1 ..."
  ./$TEST_NAME $CI_ARG --test-threads=1 
fi
