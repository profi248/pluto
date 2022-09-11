# compile and run backup coordinator

FROM lukemathwalker/cargo-chef:latest-rust-1.63.0 AS chef
WORKDIR app

FROM chef AS planner
LABEL stage=planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
LABEL stage=builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin pluto-coordinator

# We do not need the Rust toolchain to run the binary!
FROM debian:bullseye-slim AS runtime
WORKDIR app
RUN apt-get update && apt-get install libpq5 -y && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pluto-coordinator /app
COPY .env /app
ENTRYPOINT ["/app/pluto-coordinator"]
