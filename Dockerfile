FROM rust:latest AS builder
WORKDIR /build
ADD . .
RUN cargo build --all --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y wget
RUN wget https://apertium.projectjj.com/apt/install-nightly.sh && bash install-nightly.sh
RUN apt-get update && apt-get install -y divvun-gramcheck hfst
RUN apt-get update && apt-get upgrade -y
WORKDIR /app/
COPY --from=builder /build/target/release/divvun-api .
COPY deployment/config.toml .
ENV LANG C.UTF-8
VOLUME data
CMD ["./divvun-api", "-c", "config.toml"]
