FROM ghcr.io/geng-engine/cargo-geng

RUN apt-get update -y && \
  apt-get install -y pkg-config libssl-dev

COPY ./crates ./crates
COPY ./src ./src
COPY ./Cargo.toml ./
RUN cargo build --release --package ctl-server
