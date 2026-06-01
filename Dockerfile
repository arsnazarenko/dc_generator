FROM rust:1.91.1-alpine AS builder

WORKDIR /ws

RUN rustup target add x86_64-unknown-linux-musl
RUN apk add musl-dev
# RUN apk add g++ libc-dev cmake make

COPY Cargo.toml Cargo.lock ./
COPY src src
RUN cargo fetch
RUN cargo build --target x86_64-unknown-linux-musl --release

FROM alpine:latest
COPY --from=builder /ws/target/x86_64-unknown-linux-musl/release/dc-generator /usr/local/bin/dc-generator

CMD dc-generator kafka --brokers "$KAFKA_ADDRESS" --topic "$TOPIC" --rps "$RPS" --partitions "$PARTITIONS" --replicas "$REPLICAS" --connections 8 --threads 8 --servers-per-zone 10
