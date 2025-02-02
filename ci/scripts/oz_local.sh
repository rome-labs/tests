#!/bin/bash

has_container_exited() {
  local container_name="$1"
  if [ -n "$(docker ps -a --filter "name=${container_name}" --filter "status=exited" -q)" ]; then
    return 0  
  else
    return 1
  fi
}

copy_logs() {
  LOGS_DIR="../records/uniswap-proxy-docker-logs"
  mkdir -p $LOGS_DIR
  docker-compose ps -aq | xargs -I {} sh -c 'docker logs {} > ../records/uniswap-proxy-docker-logs/$(docker inspect --format="{{.Name}}" {}).log 2>&1'
}

clear_env() {
  copy_logs
  docker-compose down 
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

evm_address="0x768b73EE6CA9e0A1bc32868CA65dB89E44696DD8"

balance_check() {
  local rpc_url="$1"
  local address="$2"

  response=$(curl -s $rpc_url \
    -X POST \
    -H "Content-Type: application/json" \
    --data '{"method":"eth_getBalance","params":["'$address'", "latest"],"id":1,"jsonrpc":"2.0"}')

  balance=$(echo "$response" | sed -n 's/.*"result":"\([^"]*\)".*/\1/p')
  if [ -z "$balance" ] || [ "$balance" == "0x0" ]; then
    clear_env
    return 0
  else
    return 1
  fi
}


mkdir -p records
touch ./records/uniswap-proxy.txt
cd ./ci

docker-compose up -d solana proxy hercules rhea

airdrop

docker-compose up -d reg_rollup
until has_container_exited "reg_rollup"; do
  sleep 2
done

docker-compose up -d create_balance
until has_container_exited "create_balance"; do
  sleep 2
done


export PROXY_URL=http://localhost:9090
export NETWORK_ID=1001
export PRIVATE_KEYS=1c730a4f953f917114891048c3ca8adcb064db6ece6402623dc75a23727a253b,7cb079968e7a95afdfa9c4ef5e7074f84833cbe925da51fbcfc0b45491f5130a,4a2408284d91687081df213732338f28f39684dc886b7c1e4e750c287182f858,43a449d05b8fbd69b9a8f3707c0a27c383d47ad62f87c555e3b6179856908cc1,d191daa598a77767eae21d33c865422f95a01f705bc4fbef8271d46177b075be,dbd8ab1077d8f1c7378d3f9255863b2674087153cd311185e97c743c2783f82c,c5d7a9637f9501eaa1036b84e3a72cbf27db969eaa5d766d2207f7dc756e36d6,106df7f7b769a4b3e1e3557d76c1a5d97578c112261c1e1c98c30fb66e4cf267,bf59009b7811298522887038d36692881186d114fa482ded6047b6f5ecd6cc70,bea2a2e6c1deaba51061fa9be58725bc2531e09292a08d6e3230d1dc2450f897

export WEB3_RPC_URL=$PROXY_URL
export WEB3_PRIVATE_KEY=3f37802575d0840281551d5619256a84762e8236325537e8818730082645be65

web3 transfer 20000 to 0xe235b9caf55b58863Ae955A372e49362b0f93726
web3 transfer 20000 to 0x8870D7800Dd9B1CF34DBC54D348F70A342A4606D
web3 transfer 20000 to 0x229E93198d584C397DFc40024d1A3dA10B73aB32
web3 transfer 20000 to 0x1Db0897eDcB6F92E39b3c3d9cF633d9367c4AF32
web3 transfer 20000 to 0xaA4d6f4FF831181A2bBfD4d62260DabDeA964fF1
web3 transfer 20000 to 0x6970d087e7e78A13Ea562296edb05f4BB64D5c2E
web3 transfer 20000 to 0xD030B620B1687fC079487F7Fa1e9bd0Fbb1e12A5
web3 transfer 20000 to 0x5314Ba82B894D9b478E8920e19bdFADEb7fcD22D
web3 transfer 20000 to 0x5AD7406acB3a98C0D5a115DF0BF1C40271806DeB
web3 transfer 20000 to 0x369a08daac8E632207BFF550158b1b48cD927E19
