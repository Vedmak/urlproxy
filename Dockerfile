FROM rust:1.65 as builder
WORKDIR /app
COPY src src/
COPY Cargo.lock .
COPY Cargo.toml .
RUN cargo build --release


FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/urlproxy /app/urlproxy
CMD ["/app/urlproxy"]