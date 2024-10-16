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


COPY ./tests /opt/tests
COPY ./rome-sdk /opt/rome-sdk
COPY ./rome-evm /opt/rome-evm

RUN mkdir /opt/bin

WORKDIR /opt/tests/evm
RUN cargo rustc --test evm -- --emit link="/opt/bin/evm"
RUN cargo rustc --test state_comparison -- --emit link="/opt/bin/state_comparison"

#RUN cargo test --no-run
#RUN find target/debug/deps -name "simple_tx"* -type f -executable | xargs -I {} mv {} /opt/bin/cli/simple_tx

FROM ubuntu:20.04 AS test_env

RUN apt-get update && apt install -y openssl

COPY --from=build /opt/bin /opt/bin
COPY --from=build /opt/tests/ci /opt/ci
COPY --from=contracts /opt/ /opt/solidity/
COPY --from=solana /usr/bin/solana /opt/bin/
COPY --from=solana /usr/bin/solana-keygen /opt/bin/

ENTRYPOINT [ "/opt/ci/start_tests.sh" ]

