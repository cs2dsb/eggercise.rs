FROM ghcr.io/cs2dsb/eggercise.rs/rust/nightly:latest AS builder
WORKDIR /server

COPY . .

RUN \
    --mount=type=cache,target=/server/target,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    cargo sweep --installed && cargo sweep --time 30 