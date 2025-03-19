#!/bin/bash


export REF_NAME=fix_gas_estimate
export ROME_EVM_TAG=fix_gas_estimate
export PROXY_TAG=fix_gas_estimate
export TASKS_NUMBER=16

docker ps -aq | xargs docker stop | xargs docker rm

#docker build --tag romelabs/oz:$REF_NAME -f ../Dockerfile_oz .

docker-compose  up airdrop_oz

docker-compose  up reg_rollup

docker-compose  up deposit

docker-compose  up oz -d

docker-compose  exec -i oz /opt/bin/oz

