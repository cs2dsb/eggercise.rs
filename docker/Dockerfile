FROM ghcr.io/cs2dsb/eggercise.rs/rust/nightly-slim:latest AS runtime
RUN apt-get update && apt-get install -y openssl
WORKDIR /server
COPY ./dist/server /usr/local/bin
RUN mkdir -p /opt/server/database
COPY ./dist/assets /opt/server/assets

ENV RUST_LOG=warn,server=debug,service-worker=debug,shared=debug,sqlite_profiling=trace,sqlite_tracing=trace,sqlite=trace,[client_log]=trace
ENV BIND_ADDR=0.0.0.0
ENV PORT=9090
ENV ASSETS_DIR=/opt/server/assets
ENV SECURE_SESSIONS=true
ENTRYPOINT ["/usr/local/bin/server"]
