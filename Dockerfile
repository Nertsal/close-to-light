FROM ghcr.io/geng-engine/cargo-geng@sha256:0e57d6ddd0b82f845fc254a553d2259d21f43ae7bc2068490db6659c8ed30fbe

RUN apt-get update -y && \
  apt-get install -y pkg-config libssl-dev
