FROM ubuntu:24.04

ARG DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    software-properties-common && \
    add-apt-repository --yes universe

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential ca-certificates curl git pkg-config \
    cmake \
    clang libssl-dev musl-tools \
    bash coreutils findutils \
    just && \
    rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo
ENV PATH="/usr/local/cargo/bin:${PATH}"

RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal && \
    rustup component add clippy rustfmt && \
    rustup target add aarch64-unknown-linux-musl

RUN cargo install dotslash --locked

RUN chmod -R a+rwx /usr/local/rustup /usr/local/cargo

USER ubuntu
WORKDIR /workspace
