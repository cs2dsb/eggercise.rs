FROM ghcr.io/cs2dsb/eggercise.rs/rust/nightly-slim:latest AS runtime
RUN apt-get update && apt-get install -y openssl
WORKDIR /server
COPY ./dist/server /usr/local/bin
RUN mkdir -p /opt/server/database
COPY ./dist/assets /opt/server/assets

ENTRYPOINT ["/usr/local/bin/server"]
