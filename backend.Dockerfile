FROM ghcr.io/profiidev/images/rust-musl-watch:main@sha256:e65645871b5e1c970f7417189d9b0747dc8d0ad7bf7844ce8f394e2be64e1598

RUN apt update
RUN apt install build-essential pkg-config libssl-dev -y
