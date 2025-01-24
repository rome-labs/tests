FROM ethereum/solc:0.8.28 AS solidity
FROM ethereum/solc:0.5.16 AS solidity_5_16
FROM ubuntu:20.04 AS contracts
RUN apt-get update && \
    DEBIAN_FRONTEND=nontineractive apt-get -y install xxd && \
    rm -rf /var/lib/apt/lists/* /var/lib/apt/cache/*
COPY tests/solidity/ /opt/

# solc 0.8.0
COPY --from=solidity /usr/bin/solc /usr/bin/solc
WORKDIR /opt/
RUN solc --optimize --optimize-runs 200 --output-dir . --bin --abi *.sol && \
    for file in $(ls *.bin); do xxd -r -p $file >${file}ary; done && \
        ls -l

# solc 0.5.16
COPY --from=solidity_5_16 /usr/bin/solc /usr/bin/solc
WORKDIR /opt/uniswap
RUN solc --optimize --optimize-runs 200 --output-dir . --bin --abi *.sol && \
    for file in $(ls *.bin); do xxd -r -p $file >${file}ary; done && \
        ls -l


FROM anzaxyz/agave:v2.1.7 as solana

FROM rust:1.79.0 as build


COPY ./tests /opt/tests
COPY ./rome-sdk /opt/rome-sdk
COPY ./rome-evm /opt/rome-evm

RUN mkdir /opt/bin

WORKDIR /opt/tests/evm
RUN cargo rustc --test evm -- --emit link="/opt/bin/evm"
RUN cargo rustc --test state_comparison -- --emit link="/opt/bin/state_comparison"

#RUN cargo test --no-run
#RUN find target/debug/deps -name "simple_tx"* -type f -executable | xargs -I {} mv {} /opt/bin/cli/simple_tx

FROM ubuntu:22.04 AS test_env

RUN apt-get update && apt install -y openssl libpq-dev

COPY --from=build /opt/bin /opt/bin
COPY --from=build /opt/tests/ci /opt/ci
COPY --from=contracts /opt/ /opt/solidity/
COPY --from=solana /usr/bin/solana /opt/bin/
COPY --from=solana /usr/bin/solana-keygen /opt/bin/

ENTRYPOINT [ "/opt/ci/start_tests.sh" ]

