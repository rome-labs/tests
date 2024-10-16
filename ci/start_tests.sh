#!/bin/bash

sleep 10
/opt/bin/solana config set --url http://solana:8899
/opt/bin/solana-keygen new --no-bip39-passphrase --silent 
/opt/bin/solana airdrop -k /opt/ci/keys/test-account-keypair.json 100
/opt/bin/solana airdrop -k /opt/ci/keys/rollup-owner-keypair.json 100
/opt/bin/solana airdrop -k /opt/ci/keys/upgrade-authority-keypair.json 100

if [ -z "$TEST_NAME" ]; then
  echo "TEST_NAME is not set"
  exit 1
fi

echo "Start $TEST_NAME tests ..."
cd /opt/bin
./$TEST_NAME
