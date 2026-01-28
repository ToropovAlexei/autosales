FROM rust:1.93.0-bookworm AS builder

WORKDIR /app/backend_rust

COPY backend_rust/Cargo.toml backend_rust/Cargo.lock ./
COPY backend_rust/.sqlx ./.sqlx
COPY shared_dtos /app/shared_dtos
RUN mkdir src && echo "fn main() {}" > src/main.rs
ENV SQLX_OFFLINE=true
RUN cargo build --release
RUN rm -rf src

COPY backend_rust /app/backend_rust
RUN cargo build --release --bins
RUN cargo install sqlx-cli --no-default-features --features postgres

FROM debian:bookworm-slim

RUN apt-get update \
  && apt-get install --no-install-recommends -y ca-certificates curl libssl3 \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/backend_rust/target/release/backend_rust /usr/local/bin/backend_rust
COPY --from=builder /app/backend_rust/target/release/init /usr/local/bin/init
COPY --from=builder /app/backend_rust/target/release/seed /usr/local/bin/seed
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY backend_rust/migrations /app/migrations
COPY backend_rust/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh
RUN mkdir -p /app/uploads/images \
  && chown -R nobody:nogroup /app

USER nobody

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]
CMD ["backend_rust"]
