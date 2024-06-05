FROM ghcr.io/cs2dsb/eggercise.rs/rust/nightly:latest AS builder
WORKDIR /server

COPY . .

ARG CARGO_HOME=/cargo_home
RUN \
    --mount=type=cache,target=/server/target,sharing=locked \
    --mount=type=cache,target=/cargo_home,sharing=locked \
    cargo sweep --installed && cargo sweep --time 30 