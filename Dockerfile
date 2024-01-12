FROM ubuntu:22.04

ARG DEBIAN_FRONTEND=noninteractive
ARG CARGO_REGISTRIES_RD_TOKEN

LABEL maintainer="Deniz Hofmeister"
LABEL description="Robotics Deployment Embedded Module"

# Install dependencies required for cross-compilation
RUN apt-get update && apt-get install -y --no-install-recommends \
  wget \
  gnupg \
  software-properties-common \
  curl \
  git \
  openssh-client \
  pkg-config \
  libssl-dev \
  build-essential \
  gcc-aarch64-linux-gnu \
  g++-aarch64-linux-gnu \
  libc6-dev-arm64-cross \
  && apt-get autoremove -y \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/opt/rustup
ENV CARGO_HOME=/opt/cargo

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/opt/cargo/bin:${PATH}"

RUN chmod -R 777 /opt/rustup \
  && chmod -R 777 /opt/cargo

# Add new Rust targets for cross-compilation
RUN rustup target add x86_64-unknown-linux-gnu \
  && rustup target add aarch64-unknown-linux-gnu

COPY . /opt/rdembedded
WORKDIR /opt/rdembedded

RUN --mount=type=ssh mkdir -p -m 0700 ~/.ssh && ssh-keyscan ssh.shipyard.rs >> ~/.ssh/known_hosts
RUN --mount=type=ssh cargo login --registry rd $CARGO_REGISTRIES_RD_TOKEN
RUN --mount=type=ssh cargo build --release

RUN cp /opt/rdembedded/target/release/rdembedded /usr/bin/rdembedded && \
  mkdir -p /etc/rd && \
  cp -R /opt/rdembedded/tests/device_cfg.yaml /etc/rd/device.yaml && \
  cp -R /opt/rdembedded/tests/wireguard_cfg.yaml /etc/rd/wireguard.yaml

CMD ["bash"]
