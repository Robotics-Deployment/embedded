FROM rust:latest

ARG DEBIAN_FRONTEND=noninteractive

LABEL maintainer="Deniz Hofmeister"
LABEL description="Robotics Deployment Embedded Module"

# Install dependencies required for cross-compilation
RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu \
    libc6-dev-arm64-cross

# Add new Rust targets for cross-compilation
RUN rustup target add x86_64-unknown-linux-gnu \
    && rustup target add aarch64-unknown-linux-gnu

COPY rdembedded /opt/rdembedded
WORKDIR /opt/rdembedded

CMD ["bash"]
