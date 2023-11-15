ARG IMAGE=robotics-deployment:dev
FROM ${IMAGE}

ARG DEBIAN_FRONTEND=noninteractive

LABEL maintainer="Deniz Hofmeister"
LABEL description="Robotics Deployment Embedded Module"

# Install dependencies required for cross-compilation
RUN apt-get update && apt-get install -y --no-install-recommends \
  wget \
  gnupg \
  software-properties-common \
  curl \
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

CMD ["bash"]
