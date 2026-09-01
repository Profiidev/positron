FROM ghcr.io/profiidev/images/rust-musl-watch:main@sha256:b03ecb064002e849d0f65dfa3690053241b0bfe4b893926f527d70655ff8bedc

RUN apt update
RUN apt install build-essential pkg-config libssl-dev -y
