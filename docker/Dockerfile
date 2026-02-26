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
    clang libssl-dev libcap-dev libseccomp-dev musl-tools \
    bash coreutils findutils \
    iputils-ping netcat-openbsd openssh-client wget \
    just && \
    rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo
ENV PATH="/usr/local/cargo/bin:${PATH}"

RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal && \
    rustup component add clippy rustfmt && \
    rustup target add aarch64-unknown-linux-musl

RUN cargo install dotslash --locked
RUN cargo install cargo-insta --locked

RUN chmod -R a+rwx /usr/local/rustup /usr/local/cargo

# Add an assistant user that is also a member of the ubuntu group.
RUN if ! getent group assistant >/dev/null; then groupadd -g 1500 assistant; fi && \
    if ! id -u assistant >/dev/null 2>&1; then useradd -m -u 1500 -g 1500 -s /bin/bash assistant; fi && \
    usermod -aG ubuntu assistant

USER ubuntu
WORKDIR /workspace
