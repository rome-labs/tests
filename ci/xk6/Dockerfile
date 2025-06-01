FROM golang:1.24.0-bullseye AS builder

ENV GO111MODULE=on \
    CGO_ENABLED=1 \
    PATH="/root/go/bin:$PATH"

RUN apt-get update && apt-get install -y \
    build-essential \
    git \
    gcc \
    && rm -rf /var/lib/apt/lists/*

RUN go install go.k6.io/xk6/cmd/xk6@latest

RUN xk6 build --with github.com/distribworks/xk6-ethereum

ENV INFLUXDB_URL="http://influxdb:8086/blockspeed"
ENV TEST_SCRIPT="/scripts/examples/multiple.js"

ENTRYPOINT ["tail", "-f", "/dev/null"]
