#!/bin/bash


export REF_NAME=oz_yaml_update
export PROXY_CONFIG=/opt/cfg/proxy-config-oz.yml
export RHEA_CONFIG=/opt/cfg/rhea-config-oz.yml
export PROXY_URL=http://geth:8545

docker-compose -f docker-compose.yml up airdrop_oz
docker-compose -f docker-compose.yml up reg_rollup
docker-compose -f docker-compose.yml up deposit

docker-compose -f docker-compose.yml up proxy geth hercules rhea -d
docker-compose -f docker-compose.yml up oz -d
sleep 30
docker-compose -f docker-compose.yml exec -i oz /opt/bin/oz

