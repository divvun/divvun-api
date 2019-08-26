FROM rust:1.36 AS builder
WORKDIR /build
ADD . .
RUN cargo build --all --release

FROM debian:stretch-slim
RUN apt update
RUN apt install -y wget
RUN wget https://apertium.projectjj.com/apt/install-nightly.sh && bash install-nightly.sh
RUN apt install -y divvun-gramcheck
WORKDIR /app/
COPY --from=builder /build/target/release/divvun-api .
COPY deployment/config.toml .
VOLUME data
CMD ["./divvun-api", "-c", "config.toml"]
