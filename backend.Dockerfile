FROM ghcr.io/profiidev/images/rust-musl-watch:main@sha256:ec17dc66106621eea023280e31de6e192bbdaffc9b5730ec9bbf6cd3aec15c48

RUN apt update
RUN apt install build-essential pkg-config libssl-dev -y
