FROM ghcr.io/cs2dsb/eggercise.rs/rust/nightly:latest AS builder
WORKDIR /server
COPY rust-toolchain.toml .
# Force rustup to install the toolchain specified in rust-toolchain.toml
RUN RUSTUP_PERMIT_COPY_RENAME=false rustup show

COPY . .

# Build application
ARG CARGO_HOME=/cargo_home
RUN \
    --mount=type=cache,target=/server/target,sharing=locked \
    --mount=type=cache,target=/cargo_home,sharing=locked \
    mkdir -p $CARGO_HOME \
    && tree $CARGO_HOME \
    && cargo build --release --bin server \
    && tree $CARGO_HOME \
    && cd /server \
    && mkdir dist \
    && cd dist \
    && cp ../target/release/server . \
    && cp -r ../crates/server/assets . \
    && cd .. \
    && cargo doc \
        --no-deps \
        --workspace \
        --all-features \
        --document-private-items \
    && echo '<meta http-equiv="refresh" content="0;url=server/index.html">' | tee target/doc/index.html \
    && rm target/doc/.lock

FROM ghcr.io/cs2dsb/eggercise.rs/rust/nightly-slim:latest AS runtime
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