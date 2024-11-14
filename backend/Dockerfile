ARG TARGET=x86_64-unknown-linux-musl

FROM profidev/rust-musl-builder:latest AS chef
RUN rustup update

RUN cargo install cargo-chef 
WORKDIR /app

FROM chef AS planner
COPY backend/Cargo.toml ./Cargo.lock ./
COPY backend/src ./src
RUN cargo chef prepare

FROM chef AS builder

ARG TARGET
ENV TARGET=$TARGET

COPY --from=planner /app/recipe.json .
RUN cargo chef cook --release --target $TARGET
COPY backend/Cargo.toml ./Cargo.lock ./
COPY backend/src ./src
RUN cargo build --release --target $TARGET
RUN mv ./target/$TARGET/release/positron-backend ./app

FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

WORKDIR /app
COPY --from=builder /app/app /usr/local/bin/

CMD ["app"]