#!/bin/bash

sleep 10
/opt/bin/solana config set --url http://solana:8899
/opt/bin/solana-keygen new --no-bip39-passphrase --silent 
/opt/bin/solana airdrop -k /opt/ci/test-account-keypair.json 100
/opt/bin/solana airdrop -k /opt/ci/rome-owner-keypair.json 100

echo "Start cli tests ..."
cd /opt/bin
./cli

echo "Start rome-evm tests ..."
./evm

if [ "$CROSS_ROLLUP_TESTS" = true ]; then
    echo "Start cross rollup tests ..."
    ./cross-rollup
else
    echo "Skipping cross rollup tests ..."
fi