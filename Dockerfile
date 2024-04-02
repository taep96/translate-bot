FROM rust:slim-bookworm as builder

WORKDIR /build
COPY . .

RUN --mount=type=cache,target=/usr/local/cargo,from=rust:slim-bookworm,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    apt update && apt install pkg-config libssl-dev -y && \
    cargo build --release && \
    mv target/release/translate-bot .

FROM debian:bookworm-slim

WORKDIR /app

COPY --from=builder /build/translate-bot .
RUN apt update && apt install libssl-dev ca-certificates -y

ENV RUST_LOG=info

CMD ./translate-bot
