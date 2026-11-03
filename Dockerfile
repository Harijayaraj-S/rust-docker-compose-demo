FROM rust:1-slim-bookworm AS builder

WORKDIR /app

COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app

COPY --from=builder /app/target/release/rust-docker-compose-demo ./rust-docker-compose-demo

EXPOSE 3000

CMD ["./rust-docker-compose-demo"]
