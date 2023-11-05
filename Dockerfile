FROM robotics-deployment:dev

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

RUN wget -O - https://apt.llvm.org/llvm-snapshot.gpg.key | apt-key add -
RUN echo "deb http://apt.llvm.org/jammy/ llvm-toolchain-jammy-17 main" >> /etc/apt/sources.list && \
  echo "deb-src http://apt.llvm.org/jammy/ llvm-toolchain-jammy-17 main" >> /etc/apt/sources.list

RUN apt-get update && \
  apt-get install -y --no-install-recommends \
  llvm-17 \
  lldb-17 \
  && apt-get autoremove -y \
  && apt-get clean \
  && rm -rf /var/lib/apt/lists/*

RUN ln -s /usr/bin/lldb-vscode-17 /usr/bin/lldb-vscode

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

ENV PATH="/root/.cargo/bin:${PATH}"

# Add new Rust targets for cross-compilation
RUN rustup target add x86_64-unknown-linux-gnu \
  && rustup target add aarch64-unknown-linux-gnu \
  && rustup component add rust-analyzer

COPY . /opt/rdembedded
WORKDIR /opt/rdembedded

CMD ["bash"]
