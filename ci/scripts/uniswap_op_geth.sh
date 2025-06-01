#!/bin/bash
source ./ci/scripts/functions.sh
source ./ci/scripts/economic_tests.sh

filename=$(basename -- "$0") 
TEST_NAME="${filename%.*}"
echo "Test name: $TEST_NAME"
$(create_log_file $TEST_NAME)

evm_address="0x768b73EE6CA9e0A1bc32868CA65dB89E44696DD8"


cd ./ci
echo "Starting docker-compose..."
docker-compose up --quiet-pull -d solana proxy geth rhea hercules faucet postgres > /dev/null 2>&1
sleep 5
airdrop
regrollup
deposit
# $(get_logs_from faucet)
airdrop_to_wallets

#################
# Op Geth Tests #
#################
OPERATOR_SOL=$(get_key_of ./rhea-sender.json)
echo "[ Info ] - Operator SOL key: $OPERATOR_SOL"
FEE_RECIPIENT_RSOL=0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5
OPERATOR_BAL_BEFORE=$(get_solana_balance "$OPERATOR_SOL")
FEE_RECIPIENT_BAL_BEFORE=$(get_eth_balance "$FEE_RECIPIENT_RSOL")
# OPERATOR_SOL=4X2seLbitEeXQXwQveTz3hFX4wNMYqgaRgB37VzS9QZY
# FEE_RECIPIENT_RSOL=0x95222290DD7278Aa3Ddd389Cc1E1d165CC4BAfe5


echo "Starting Op Geth tests..."
# Check balance
if balance_check "http://127.0.0.1:9090" $evm_address 0; then
  echo "Insufficient proxy balance, exiting..."
  exit 1
fi

# Check if an argument was provided
docker pull romelabs/uniswap-v2-core:${UNISWAP_V2_TAG:-latest} > /dev/null 2>&1

if [ -n "$1" ]; then
  SUIT=$1
  docker run --rm --network="ci_net" --name="uniswap" \
    -e NETWORK='op-geth' \
    -e CHAIN_ID='1001' \
    romelabs/uniswap-v2-core:${UNISWAP_V2_TAG:-latest} \
    yarn test --grep "$SUIT" | tee ../records/$TEST_NAME$SUIT.txt
else
  docker run --rm --network="ci_net" --name="uniswap" \
    -e NETWORK='op-geth' \
    -e CHAIN_ID='1001' \
    romelabs/uniswap-v2-core:${UNISWAP_V2_TAG:-latest} \
    yarn test | tee ../records/$TEST_NAME.txt
fi

OPERATOR_BAL_AFTER=$(get_solana_balance "$OPERATOR_SOL")
FEE_RECIPIENT_BAL_AFTER=$(get_eth_balance "$FEE_RECIPIENT_RSOL")
echo "========================= ECONOMIC TESTS ========================="
echo "[ Info ] - Balances in lamports"
echo "[ Info ] - OPERATOR_BAL_BEFORE_: $OPERATOR_BAL_BEFORE"
echo "[ Info ] - OPERATOR_BAL_AFTER__: $OPERATOR_BAL_AFTER"
echo "[ Info ] - FEE_RECIPIENT_BEFORE: $FEE_RECIPIENT_BAL_BEFORE"
echo "[ Info ] - FEE_RECIPIENT_AFTER_: $FEE_RECIPIENT_BAL_AFTER"
echo "[ Info ] - Before - After is: OPERATOR_BAL_BEFORE + FEE_RECIPIENT_BAL_BEFORE - OPERATOR_BAL_AFTER - FEE_RECIPIENT_BAL_AFTER:"
echo "[ Info ] - Before - After => $(echo "$OPERATOR_BAL_BEFORE + $FEE_RECIPIENT_BAL_BEFORE - $OPERATOR_BAL_AFTER - $FEE_RECIPIENT_BAL_AFTER" | bc) "
OPERATOR_BAL_CHANGE=$(echo "$OPERATOR_BAL_BEFORE - $OPERATOR_BAL_AFTER" | bc)
FEE_RECIPIENT_BAL_CHANGE=$(echo "$FEE_RECIPIENT_BAL_AFTER - $FEE_RECIPIENT_BAL_BEFORE" | bc)
echo "[ Info ] - $OPERATOR_BAL_CHANGE < OPERATOR | OPERATOR_BAL_BEFORE - OPERATOR_BAL_AFTER"
echo "[ Info ] - $FEE_RECIPIENT_BAL_CHANGE < FEE_RECIPIENT | FEE_RECIPIENT_BAL_AFTER - FEE_RECIPIENT_BAL_BEFORE"
if [ "$(echo "$OPERATOR_BAL_CHANGE == $FEE_RECIPIENT_BAL_CHANGE" | bc -l)" -eq 1 ]; then
    echo "[ Info ] - Balances are equal"
    echo "========================= -------------- ========================="
else
    echo "[ Warning ] - Balances are different!"
    echo "========================= -------------- ========================="
    # clear_env
    # exit 1
fi

if cat ../records/$TEST_NAME$SUIT.txt | grep -E '[1-9][0-9]? failing'; then
  echo "Tests failed. Exiting with error."
  clear_env
  exit 1
else
  echo "Tests passed. Stopping tests..."
  clear_env
fi
