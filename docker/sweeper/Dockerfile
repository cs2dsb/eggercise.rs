FROM ghcr.io/cs2dsb/eggercise.rs/rust/nightly:latest AS builder
WORKDIR /server

COPY . .

ARG CARGO_HOME=/server/target/cargo_home
RUN \
    --mount=type=cache,target=/server/target,sharing=locked \
    cargo sweep --installed && cargo sweep --time 30 