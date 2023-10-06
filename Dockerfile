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
    gdb  \
    pkg-config  \
    git  \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get autoremove -y \
    && apt-get clean

# Install Rust and Cargo
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Add Cargo's bin directory to PATH
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy your asd directory into the image
COPY rdembedded /opt/rdembedded

# Set the working directory
WORKDIR /opt/rdembedded

# Default command
CMD ["bash"]
