FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /server
COPY rust-toolchain.toml .
# Force rustup to install the toolchain specified in rust-toolchain.toml
RUN rustup show

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /server/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN \
    --mount=type=cache,target=/server/target,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json \
        --package server
RUN \
    --mount=type=cache,target=/server/target,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json \
        --target wasm32-unknown-unknown \
        --package service-worker \
        --package client 
# Have to be copied after cook or cook overwrites the bin
COPY . .
# Build application
RUN \
    --mount=type=cache,target=/server/target,sharing=locked \
    cargo build --release --bin server \
    && cd /server && mkdir dist && cd dist && cp ../target/release/server . && cp -r ../crates/server/assets .

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y openssl
WORKDIR /server
COPY --from=builder /server/dist/server /usr/local/bin
RUN mkdir -p /opt/server
COPY --from=builder /server/dist/assets /opt/server/assets
# COPY --from=builder /server/target /opt/target

ENV RUST_LOG=warn,server=debug,service-worker=debug,shared=debug,sqlite_profiling=trace,sqlite_tracing=trace,sqlite=trace
ENV BIND_ADDR=0.0.0.0
ENV PORT=9090
ENV ASSETS_DIR=/opt/server/assets
ENTRYPOINT ["/usr/local/bin/server"]