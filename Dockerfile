FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /server

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /server/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin server

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /server
COPY --from=builder /server/target/release/server /usr/local/bin
RUN mkdir -p /opt/server
COPY --from=builder /server/assets /opt/server/

ENV RUST_LOG=warn,server=debug,service-worker=debug,shared=debug,sqlite_profiling=trace,sqlite_tracing=trace,sqlite=trace
ENV BIND_ADDR=0.0.0.0
ENV PORT=9090
ENTRYPOINT ["/usr/local/bin/server"]