FROM rust:alpine3.24 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/dota-data-generator /usr/local/bin/
ENTRYPOINT ["/usr/local/bin/dota-data-generator"]