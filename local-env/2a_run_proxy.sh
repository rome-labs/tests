#!/bin/bash

docker-compose up -d proxy
echo "Wait 10 seconds..."
sleep 10
solana -u http://localhost:8899 airdrop 100 ./proxy-sender.json

