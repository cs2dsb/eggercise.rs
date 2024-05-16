FROM rustlang/rust:nightly AS builder
WORKDIR /server
COPY rust-toolchain.toml .
# Force rustup to install the toolchain specified in rust-toolchain.toml
RUN rustup show

COPY . .

# Build application
RUN \
    --mount=type=cache,target=/server/target,sharing=locked \
    cargo build --release --bin server \
    && cd /server && mkdir dist && cd dist && cp ../target/release/server . && cp -r ../crates/server/assets .

FROM rustlang/rust:nightly-slim AS runtime
RUN apt-get update && apt-get install -y openssl
WORKDIR /server
COPY --from=builder /server/dist/server /usr/local/bin
RUN mkdir -p /opt/server
COPY --from=builder /server/dist/assets /opt/server/assets

ENV RUST_LOG=warn,server=debug,service-worker=debug,shared=debug,sqlite_profiling=trace,sqlite_tracing=trace,sqlite=trace
ENV BIND_ADDR=0.0.0.0
ENV PORT=9090
ENV ASSETS_DIR=/opt/server/assets
ENTRYPOINT ["/usr/local/bin/server"]