ARG IMAGE=rust:latest

FROM ${IMAGE}

ARG DEBIAN_FRONTEND=noninteractive

LABEL maintainer="Deniz Hofmeister"
LABEL description="Robotics Deployment Embedded Module"

COPY rdembedded /opt/rdembedded
WORKDIR /opt/rdembedded

CMD ["bash"]