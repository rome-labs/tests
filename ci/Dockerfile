# syntax=docker/dockerfile:1.4

# -------- Stage: Solidity Compilation (multiple versions) --------
FROM ethereum/solc:0.8.28 AS solidity
FROM ethereum/solc:0.5.16 AS solidity_5_16

FROM ubuntu:24.04 AS contracts
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt/lists,sharing=locked \
    apt-get update -qq && DEBIAN_FRONTEND=noninteractive apt-get install -y -qq \
    xxd curl

COPY tests/solidity/ /opt/
COPY tests/solidity/spl/ /opt/

COPY --from=solidity /usr/bin/solc /usr/bin/solc
WORKDIR /opt/
RUN solc --optimize --optimize-runs 200 --output-dir . --bin --abi *.sol && \
    find . -name '*.bin' -exec sh -c 'for f; do xxd -r -p "$f" > "${f}ary"; done' sh {} +

COPY --from=solidity_5_16 /usr/bin/solc /usr/bin/solc
WORKDIR /opt/uniswap
RUN solc --optimize --optimize-runs 200 --output-dir . --bin --abi *.sol && \
    find . -name '*.bin' -exec sh -c 'for f; do xxd -r -p "$f" > "${f}ary"; done' sh {} +

# -------- Stage: Solana CLI --------
    FROM anzaxyz/agave:v2.2.14 AS solana
    RUN apt-get update -qq && apt-get install -qq -y --no-install-recommends \
        libudev-dev pkg-config curl build-essential libssl-dev && \
        rm -rf /var/lib/apt/lists/*
    
    RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal && \
        . "$HOME/.cargo/env" && \
        cargo install spl-token-cli && \
        which spl-token

# -------- Stage: Rust Build --------
FROM rust:1.86.0-slim as build
ENV CARGO_HOME=/usr/local/cargo
ENV CARGO_TARGET_DIR=/opt/tests/target

RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt/lists,sharing=locked \
    apt-get update -qq && apt-get install -qq -y \
    protobuf-compiler pkg-config libssl-dev perl libpq-dev build-essential

# Step 2
WORKDIR /opt/tests/

# Copy real source code
COPY ./tests /opt/tests
COPY ./rome-sdk /opt/rome-sdk
COPY ./rome-relayer /opt/rome-relayer
COPY ./rome-evm /opt/rome-evm

# Build
RUN --mount=type=cache,target=/usr/local/cargo/registry,id=cargo-registry \
    --mount=type=cache,target=/usr/local/cargo/git,id=cargo-git \
    --mount=type=cache,target=/opt/tests/target,id=target-cache \
    cargo test --test evm --no-run --locked --release && \
    cargo test --test spl --no-run --locked --release && \
    cargo test --test state_comparison --no-run --locked --release && \
    cargo test --test state_comparison_2 --no-run --locked --release && \
    mkdir -p /opt/bin && \
    ls && \
    ls /opt/tests/target/ && \
    for bin in spl evm state_comparison state_comparison_2; do \
      f=$(find /opt/tests/target/release/deps -maxdepth 1 -type f -executable -name "${bin}-*" | head -n1) && \
      cp "$f" "/opt/bin/$bin"; \
    done

RUN echo "Built binaries:" && ls -lh /opt/bin

# -------- Final Stage --------
FROM ubuntu:24.04 AS test_env
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt/lists,sharing=locked \
    apt-get update -qq && apt-get install -qq -y --no-install-recommends \
    openssl libpq-dev

COPY --from=build /opt/bin /opt/bin
COPY --from=build /opt/tests/ci /opt/ci
COPY --from=contracts /opt/ /opt/solidity/
COPY --from=solana /usr/bin/solana /opt/bin/
COPY --from=solana /usr/bin/solana-keygen /opt/bin/
COPY --from=solana /root/.cargo/bin/spl-token /opt/bin/

ENTRYPOINT [ "/opt/ci/start_tests.sh" ]
