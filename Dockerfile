FROM ghcr.io/geng-engine/cargo-geng

RUN apt-get update -y && \
  apt-get install -y pkg-config libssl-dev
