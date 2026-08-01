FROM ghcr.io/profiidev/images/rust-musl-watch:main@sha256:a82703d3509314ca08c665c921114c9b0cbee2c464b2a038e39c47e9eef05f9a

RUN apt update
RUN apt install build-essential pkg-config libssl-dev -y
