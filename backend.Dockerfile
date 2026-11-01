FROM ghcr.io/profiidev/images/rust-musl-watch:main@sha256:75520ac615ecee80262bde9954e9c440db9be2d9aecd3769158b43e664c765d0

RUN apt update
RUN apt install build-essential pkg-config libssl-dev -y
