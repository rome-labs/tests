# Use Solidity compilers for different versions
FROM ethereum/solc:0.8.28 AS solidity
FROM ethereum/solc:0.5.16 AS solidity_5_16

FROM ubuntu:20.04 AS contracts  

RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get -y install apt-utils xxd && \
    rm -rf /var/lib/apt/lists/* /var/lib/apt/cache/*

COPY tests/solidity/ /opt/

# ---- Solidity 0.8.0 Compilation ----
COPY --from=solidity /usr/bin/solc /usr/bin/solc
WORKDIR /opt/

RUN if [ -d "/tmp/solidity_cache" ]; then \  
        echo "Using cached Solidity compilation"; \
        cp -r /tmp/solidity_cache/* . ; \
    else \  
        solc --optimize --optimize-runs 200 --output-dir . --bin --abi *.sol && \
        for file in $(ls *.bin); do xxd -r -p $file >${file}ary; done && \
        ls -l && \
        mkdir -p /tmp/solidity_cache && cp -r * /tmp/solidity_cache; \
    fi

# ---- Solidity 0.5.16 Compilation ----
COPY --from=solidity_5_16 /usr/bin/solc /usr/bin/solc
WORKDIR /opt/uniswap

RUN if [ -d "/tmp/uniswap_solidity_cache" ]; then \  
        echo "Using cached Uniswap Solidity compilation"; \
        cp -r /tmp/uniswap_solidity_cache/* . ; \
    else \  
        solc --optimize --optimize-runs 200 --output-dir . --bin --abi *.sol && \
        for file in $(ls *.bin); do xxd -r -p $file >${file}ary; done && \
        ls -l && \
        mkdir -p /tmp/uniswap_solidity_cache && cp -r * /tmp/uniswap_solidity_cache; \
    fi

FROM anzaxyz/agave:v2.1.7 as solana

FROM rust:1.79.0 as build


COPY ./tests /opt/tests
COPY ./rome-sdk /opt/rome-sdk
COPY ./rome-evm /opt/rome-evm

RUN mkdir /opt/bin

WORKDIR /opt/tests/evm
RUN cargo rustc --test evm -- --emit link="/opt/bin/evm"
RUN cargo rustc --test state_comparison -- --emit link="/opt/bin/state_comparison"

FROM ubuntu:22.04 AS test_env

RUN apt-get update && apt install -y openssl libpq-dev

COPY --from=build /opt/bin /opt/bin
COPY --from=build /opt/tests/ci /opt/ci
COPY --from=contracts /opt/ /opt/solidity/
COPY --from=solana /usr/bin/solana /opt/bin/
COPY --from=solana /usr/bin/solana-keygen /opt/bin/

ENTRYPOINT [ "/opt/ci/start_tests.sh" ]
