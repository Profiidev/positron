ARG TARGET=x86_64-unknown-linux-musl

FROM ghcr.io/profiidev/images/rust-musl-builder@sha256:250f3e12cbacbd71a5eaa1ebce624c2d931d9bc7488b3d8d51ce0867b1803f35 AS planner

COPY backend/entity/Cargo.toml ./entity/Cargo.toml
COPY backend/migration/Cargo.toml ./migration/Cargo.toml
COPY backend/Cargo.toml ./Cargo.lock ./

RUN echo "[workspace]" >> Cargo.toml && \
    echo "members = [\"entity\", \"migration\"]" >> Cargo.toml

RUN cargo chef prepare --recipe-path recipe.json --bin positron-backend

FROM ghcr.io/profiidev/images/rust-musl-builder@sha256:250f3e12cbacbd71a5eaa1ebce624c2d931d9bc7488b3d8d51ce0867b1803f35 AS builder

ARG TARGET
ENV TARGET=$TARGET

COPY --from=planner /app/recipe.json .

RUN cargo chef cook --release --target $TARGET

COPY backend/src ./src
COPY backend/entity ./entity
COPY backend/migration ./migration
COPY backend/Cargo.toml ./Cargo.lock ./

RUN cargo build --release --target $TARGET --bin positron-backend
RUN mv ./target/$TARGET/release/positron-backend ./app

FROM alpine

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

WORKDIR /app
COPY --from=builder /app/app /usr/local/bin/

CMD ["app"]