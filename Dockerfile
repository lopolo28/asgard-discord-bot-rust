FROM rust:1.66.0 as builder
WORKDIR /usr/src/myapp
COPY . .
RUN cargo install --path .
 
FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/myapp /usr/local/bin/myapp
CMD ["myapp"]