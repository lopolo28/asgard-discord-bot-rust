FROM rust:1.66.0 as builder
WORKDIR /usr/src/asgard-discord-bot-rust
COPY . .
RUN cargo install --path .
 
FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/asgard-discord-bot-rust /usr/local/bin/asgard-discord-bot-rust
CMD ["asgard-discord-bot-rust"]