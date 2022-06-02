# compile and run backup coordinator

FROM rust:1.61 as builder
WORKDIR /usr/src/pluto-coordinator
COPY . .
RUN cargo install --bin pluto-coordinator --path coordinator

FROM debian:buster-slim
RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/pluto-coordinator /usr/local/bin/pluto-coordinator
CMD ["pluto-coordinator"]
