ARG TARGET=x86_64-unknown-linux-gnu
ARG RUSTFLAGS="-C target-feature=+crt-static"

# only tmp when webauthn-rs removes openssl dependency
FROM ghcr.io/profiidev/images/rust-gnu-builder:main AS chef

RUN apt update
RUN apt install build-essential pkg-config libssl-dev -y

FROM chef AS planner

ARG TARGET
ARG RUSTFLAGS

COPY backend/entity/Cargo.toml ./entity/Cargo.toml
COPY backend/migration/Cargo.toml ./migration/Cargo.toml
COPY backend/Cargo.toml ./Cargo.lock ./

RUN echo "[workspace]" >> Cargo.toml && \
    echo "members = [\"entity\", \"migration\"]" >> Cargo.toml

RUN cargo chef prepare --recipe-path recipe.json --bin backend

FROM chef AS builder

ARG TARGET
ARG RUSTFLAGS

COPY --from=planner /app/recipe.json .

RUN cargo chef cook --release --target $TARGET

COPY backend/src ./src
COPY backend/entity ./entity
COPY backend/migration ./migration
COPY backend/Cargo.toml ./Cargo.lock ./

RUN cargo build --release --target $TARGET --bin backend
RUN mv ./target/$TARGET/release/backend ./app

FROM alpine

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

WORKDIR /app
COPY --from=builder /app/app /usr/local/bin/

CMD ["app"]