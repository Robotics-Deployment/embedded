# Use Ubuntu 22.04 as the base image
FROM ubuntu:22.04

# Set environment variables to non-interactive (this prevents some prompts)
ARG DEBIAN_FRONTEND=noninteractive

# Metadata as described in your original Dockerfile
LABEL maintainer="Deniz Hofmeister"
LABEL description="Robotics Deployment Heartbeat Transmitter"

# Install prerequisites and utilities
RUN apt-get update && \
    apt-get install -y --no-install-recommends  \
    curl  \
    build-essential  \
    gcc-x86-64-linux-gnu \
    g++-x86-64-linux-gnu \
    libc6-dev-amd64-cross \
    gcc-aarch64-linux-gnu \
    g++-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    gdb  \
    pkg-config  \
    git  \
    ca-certificates  \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get autoremove -y \
    && apt-get clean

# Install Rust and Cargo
RUN curl https://sh.rustup.rs -sSf | \
        sh -s -- --default-toolchain stable -y

# Add Cargo's bin directory to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Install the Rust x86_64 target
RUN rustup target add x86_64-unknown-linux-gnu
RUN rustup toolchain install stable-x86_64-unknown-linux-gnu

ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc \
    CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc \
    CXX_x86_64_unknown_linux_gnu=x86_64-linux-gnu-g++

# Install the Rust aarch64 target
RUN rustup target add aarch64-unknown-linux-gnu
RUN rustup toolchain install stable-aarch64-unknown-linux-gnu

ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++

# Copy your asd directory into the image
COPY rdembedded /opt/rdembedded

# Set the working directory
WORKDIR /opt/rdembedded

# Default command
CMD ["bash"]
