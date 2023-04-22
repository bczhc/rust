FROM ubuntu

COPY / /rust/
WORKDIR /

# Install requirements and dependencies
RUN apt update && \
    export DEBIAN_FRONTEND=noninteractive && \
    apt install -y libudev-dev curl gcc libssl-dev pkg-config libclang-dev libsqlite3-dev libxdo-dev

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs > installer && \
    chmod +x installer && \
    ./installer -y --default-toolchain nightly && \
    . ~/.cargo/env && \
    rustc --version

WORKDIR /rust/

# Build
RUN . ~/.cargo/env && \
    cargo build --release

# Test
RUN . ~/.cargo/env && \
    cargo test --release
