FROM rust:1.81-slim-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app

RUN adduser book && chown -R book /app
USER book

COPY --from=builder /app/target/release/api ./target/release/api

ENV PORT 9000
EXPOSE $PORT
ENTRYPOINT ["./target/release/api"]
