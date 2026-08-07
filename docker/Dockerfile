FROM ubuntu:24.04

ARG DEBIAN_FRONTEND=noninteractive
ARG NODE_VERSION=22.22.0

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    software-properties-common && \
    add-apt-repository --yes universe

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential ca-certificates curl git pkg-config \
    cmake \
    clang libssl-dev libcap-dev libseccomp-dev musl-tools \
    bubblewrap \
    iproute2 \
    jq \
    fd-find \
    ripgrep \
    python3 \
    xz-utils \
    bash coreutils findutils \
    iputils-ping netcat-openbsd openssh-client wget \
    sudo \
    just && \
    rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo
ENV PATH="/usr/local/cargo/bin:${PATH}"

RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal && \
    rustup component add clippy rustfmt && \
    rustup target add aarch64-unknown-linux-musl

RUN arch="$(dpkg --print-architecture)" && \
    case "$arch" in \
        amd64) node_arch="x64" ;; \
        arm64) node_arch="arm64" ;; \
        *) echo "unsupported architecture for Node.js: $arch" >&2; exit 1 ;; \
    esac && \
    curl -fsSLO "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-${node_arch}.tar.xz" && \
    tar -xJf "node-v${NODE_VERSION}-linux-${node_arch}.tar.xz" -C /usr/local --strip-components=1 && \
    rm "node-v${NODE_VERSION}-linux-${node_arch}.tar.xz"

RUN cargo install dotslash --locked
RUN cargo install cargo-insta --locked

RUN chmod -R a+rwx /usr/local/rustup /usr/local/cargo

# Add an assistant user that is also a member of the ubuntu group.
RUN if ! getent group assistant >/dev/null; then groupadd -g 1500 assistant; fi && \
    if ! id -u assistant >/dev/null 2>&1; then useradd -m -u 1500 -g 1500 -s /bin/bash assistant; fi && \
    usermod -aG ubuntu assistant

# Allow the default container user to switch to the assistant worker user without a password.
RUN echo "ubuntu ALL=(assistant:assistant) NOPASSWD: ALL" > /etc/sudoers.d/codex-worker-user && \
    chmod 0440 /etc/sudoers.d/codex-worker-user

USER ubuntu
WORKDIR /workspace
