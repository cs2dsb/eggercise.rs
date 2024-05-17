FROM rustlang/rust:nightly AS builder
WORKDIR /server
COPY rust-toolchain.toml .
# Force rustup to install the toolchain specified in rust-toolchain.toml
RUN RUSTUP_PERMIT_COPY_RENAME=false rustup show
RUN cargo install cargo-sweep
RUN rm -rf $CARGO_HOME/registry/*