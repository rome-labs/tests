FROM ethereum/solc:0.8.0 AS solidity
FROM ubuntu:20.04 AS contracts
RUN apt-get update && \
    DEBIAN_FRONTEND=nontineractive apt-get -y install xxd && \
    rm -rf /var/lib/apt/lists/* /var/lib/apt/cache/*
COPY tests/solidity/*.sol /opt/
COPY --from=solidity /usr/bin/solc /usr/bin/solc
WORKDIR /opt/
RUN solc --optimize --optimize-runs 200 --output-dir . --bin --abi *.sol && \
    for file in $(ls *.bin); do xxd -r -p $file >${file}ary; done && \
        ls -l

FROM solanalabs/solana:v1.17.28 as solana

FROM solanalabs/rust:1.75.0 as build

COPY tests/ci /opt/ci
RUN mkdir /opt/tests
COPY tests/evm /opt/tests/evm
COPY tests/Cargo.toml /opt/tests/
COPY tests/Cargo.lock /opt/tests/

COPY ./rome-sdk /opt/rome-sdk
COPY ./rome-evm-private /opt/rome-evm-private

RUN mkdir /opt/bin

ENV CHAIN_ID=1001
ENV CONTRACT_OWNER=8q76RPN5Tm6thVoQAUFhUP2diddGgtDLA6B6eShSazB2
ENV CROSS_ROLLUP_TESTS=false

WORKDIR /opt/tests/evm
RUN cargo rustc --test cli -- --emit link="/opt/bin/cli"
RUN cargo rustc --test evm -- --emit link="/opt/bin/evm"
RUN cargo rustc --test cross-rollup -- --emit link="/opt/bin/cross-rollup"

#RUN cargo test --no-run
#RUN find target/debug/deps -name "simple_tx"* -type f -executable | xargs -I {} mv {} /opt/bin/cli/simple_tx

FROM ubuntu:20.04 AS test_env

RUN apt-get update && apt install -y openssl

COPY --from=build /opt/bin /opt/bin
COPY --from=build /opt/ci /opt/ci
COPY --from=contracts /opt/ /opt/solidity/
COPY --from=solana /usr/bin/solana /opt/bin/
COPY --from=solana /usr/bin/solana-keygen /opt/bin/

ENTRYPOINT [ "/opt/ci/start_tests.sh" ]

