FROM ghcr.io/profiidev/images/rust-musl-watch:main@sha256:835dc0aeec70be66e78132969d1cf485a9b843a4e3204aab4426b5f6e034c931

RUN apt update
RUN apt install build-essential pkg-config libssl-dev -y
