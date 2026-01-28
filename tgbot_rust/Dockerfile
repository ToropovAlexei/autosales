FROM rust:1.93.0-bookworm AS builder

WORKDIR /app/tgbot_rust

COPY tgbot_rust/Cargo.toml tgbot_rust/Cargo.lock ./
COPY tgbot_rust/rust-toolchain.toml ./
COPY shared_dtos /app/shared_dtos
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

COPY tgbot_rust /app/tgbot_rust
RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install --no-install-recommends -y ca-certificates \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/tgbot_rust/target/release/tgbot_rust /usr/local/bin/tgbot_rust

RUN mkdir -p /app/logs \
  && chown -R nobody:nogroup /app

USER nobody

CMD ["tgbot_rust"]
