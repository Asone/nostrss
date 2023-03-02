FROM lukemathwalker/cargo-chef:latest-rust-1.66.0 AS chef
WORKDIR /app

FROM chef AS planner
COPY . /www/app/
WORKDIR /www/app/
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /www/app/recipe.json /www/app/recipe.json
WORKDIR /www/app/
# Build dependencies - this is the caching Docker layer!
RUN RUST_BACKTRACE=full cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . /www/app/
RUN cargo build --release --bin nostrss

# We do not need the Rust toolchain to run the binary!
FROM debian:buster-slim AS runtime
RUN apt update && apt install -y libpq-dev curl bash
WORKDIR /www/app/
COPY --from=builder /www/app/target/release/nostrss /www/app/target/release/nostrss
ENTRYPOINT ["/www/app/target/release/nostrss"]
