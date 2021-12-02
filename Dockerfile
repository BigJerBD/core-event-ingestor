FROM rust:1.56 as builder
WORKDIR /code/
COPY . .
RUN cargo build --release && cargo install --path .


FROM debian:stable-slim

RUN apt-get update \
 && apt-get install -y openssl \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/local/cargo/bin/event-ingestor /usr/local/bin/

ENTRYPOINT ["event-ingestor"]