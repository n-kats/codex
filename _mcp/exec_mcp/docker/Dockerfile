FROM ubuntu:24.04

ARG DEBIAN_FRONTEND=noninteractive
ARG RUST_TOOLCHAIN=1.93.0
ARG NODE_VERSION=22.22.0

ENV PYTHONDONTWRITEBYTECODE=1 \
    PYTHONUNBUFFERED=1 \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:${PATH}

# Avoid writing build artifacts under /tmp (which may be mounted read-only or with mismatched
# ownership in some MCP/dev-env setups).
ENV CARGO_TARGET_DIR=/var/cache/codex/target

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    build-essential ca-certificates curl git pkg-config \
    cmake \
    clang libssl-dev libcap-dev libseccomp-dev musl-tools \
    bubblewrap \
    iproute2 \
    python3 python3-pip python3-venv \
    xz-utils \
    bash coreutils findutils && \
    rm -rf /var/lib/apt/lists/*

RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal && \
    rustup toolchain install "${RUST_TOOLCHAIN}" --profile minimal --component clippy --component rustfmt && \
    rustup default "${RUST_TOOLCHAIN}" && \
    rustup toolchain install nightly --profile minimal --component rustfmt && \
    chmod -R a+rwx /usr/local/rustup /usr/local/cargo

RUN arch="$(dpkg --print-architecture)" && \
    case "$arch" in \
        amd64) node_arch="x64" ;; \
        arm64) node_arch="arm64" ;; \
        *) echo "unsupported architecture for Node.js: $arch" >&2; exit 1 ;; \
    esac && \
    curl -fsSLO "https://nodejs.org/dist/v${NODE_VERSION}/node-v${NODE_VERSION}-linux-${node_arch}.tar.xz" && \
    tar -xJf "node-v${NODE_VERSION}-linux-${node_arch}.tar.xz" -C /usr/local --strip-components=1 && \
    rm "node-v${NODE_VERSION}-linux-${node_arch}.tar.xz"

RUN if ! getent group 1000 >/dev/null; then groupadd -g 1000 ubuntu; fi && \
    if ! id -u ubuntu >/dev/null 2>&1; then useradd -m -u 1000 -g 1000 -s /bin/bash ubuntu; fi && \
    mkdir -p /app /home/ubuntu /var/cache/codex/target && \
    chown -R 1000:1000 /app /home/ubuntu /var/cache/codex

RUN python3 -m venv /opt/venv && \
    /opt/venv/bin/pip install --no-cache-dir --upgrade pip && \
    /opt/venv/bin/pip install --no-cache-dir "fastmcp<3"

ENV VIRTUAL_ENV=/opt/venv
ENV PATH="/opt/venv/bin:${PATH}"

RUN python -c "from fastmcp import FastMCP; _=FastMCP('smoke'); print('fastmcp ok')" && \
    cargo --version && \
    rustup show

EXPOSE 9765

ENTRYPOINT ["python", "/app/server.py"]
