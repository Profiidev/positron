FROM ghcr.io/profiidev/images/rust-musl-watch:main@sha256:fe2632f5dd048025fb3713ac610a06733f3c151987060230317ab4397682f312

RUN apt update
RUN apt install build-essential pkg-config libssl-dev -y
