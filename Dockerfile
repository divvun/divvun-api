FROM rust:1.36 AS builder
WORKDIR /build
ADD . .
RUN cargo build --all --release

FROM debian:stretch-slim
RUN apt-get update && apt-get install -y wget
RUN wget https://apertium.projectjj.com/apt/install-nightly.sh && bash install-nightly.sh
RUN apt-get update && apt-get install -y divvun-gramcheck hfst
WORKDIR /app/
COPY --from=builder /build/target/release/divvun-api .
COPY deployment/config.toml .
VOLUME data
CMD ["./divvun-api", "-c", "config.toml"]
