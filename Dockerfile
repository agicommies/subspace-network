# An example on how to build:
# $ docker build -t subspace . --platform linux/x86_64 -f Dockerfile

FROM debian:12-slim AS builder

WORKDIR /subspace
# Disables any interactive prompts.
ARG DEBIAN_FRONTEND=noninteractive

COPY . .

# Dependencies using during the build stage.
RUN apt update && apt install -y --no-install-recommends \
    ca-certificates \
    curl \
    build-essential \
    protobuf-compiler \
    libclang-dev \
    git

ENV PATH=/root/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin

# Installs rust with a minimal footprint and adds the WASM chain. 
RUN curl https://sh.rustup.rs -sSf | \
    sh -s -- -y --profile=minimal --default-toolchain=nightly-2024-02-01

RUN cargo build -p node-subspace --release --locked

FROM debian:12-slim

WORKDIR /subspace
# Enable extensive backtraces
ENV RUST_BACKTRACE=1

COPY --from=builder /subspace/target/release/node-subspace /subspace/node-subspace

ENTRYPOINT ["/subspace/node-subspace"]
