FROM rust:1.86-slim

SHELL ["bash", "-c"]

RUN apt-get update && apt-get install -y \
    pkg-config \
    protobuf-compiler \
    clang \
    make \
    jq \
    curl

# Install Linera tools separately to reduce memory pressure
RUN cargo install --locked linera-storage-service@0.15.8
RUN cargo install --locked linera-service@0.15.8

# Add WASM target for building Linera contracts
RUN rustup target add wasm32-unknown-unknown

# Install Node.js for frontend (using Debian's native package - no external DNS needed)
# Provides Node.js 18.20.4 which is perfect for Next.js 16.0.7
RUN apt-get install -y nodejs npm \
    && npm install -g http-server

WORKDIR /build

HEALTHCHECK CMD ["curl", "-s", "http://localhost:5173"]

ENTRYPOINT bash /build/run.bash
