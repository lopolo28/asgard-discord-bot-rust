FROM rust:1.75.0 as builder
WORKDIR /usr/src/asgard-discord-bot-rust
COPY . .
RUN cargo install --path .
 
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y openssl ca-certificates
COPY --from=builder /usr/local/cargo/bin/asgard-discord-bot-rust /usr/local/bin/asgard-discord-bot-rust
CMD ["asgard-discord-bot-rust"]
