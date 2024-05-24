FROM ghcr.io/geng-engine/cargo-geng
COPY ./crates ./crates
COPY ./src ./src
COPY ./Cargo.toml ./
RUN cargo build --release --package ctl-server
