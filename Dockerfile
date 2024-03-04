FROM rust:1.75.0-bookworm as builder

WORKDIR /usr/src/ord

# Create ~/bin directory
RUN mkdir -p ~/bin

# Download and extract just to ~/bin/just
RUN curl --proto '=https' --tlsv1.2 -sSf https://just.systems/install.sh | bash -s -- --to ~/bin

# Add ~/bin to the PATH
ENV PATH="/root/bin:${PATH}"

COPY . .

RUN cargo build --bin ord --release

FROM debian:bookworm-slim

# Install Rust, build-essential, pkg-config, libssl-dev, and ripgrep
RUN apt-get update && apt-get install -y curl build-essential pkg-config libssl-dev ripgrep && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    . $HOME/.cargo/env

COPY --from=builder /usr/src/ord/target/release/ord /usr/local/bin
COPY --from=builder /root/bin/just /usr/local/bin

RUN apt-get update && apt-get install -y openssl

ENV RUST_BACKTRACE=1
ENV RUST_LOG=info