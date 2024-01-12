#!/bin/bash

docker buildx build --ssh default=$SSH_AUTH_SOCK --build-arg CARGO_REGISTRIES_RD_TOKEN=${CARGO_REGISTRIES_RD_TOKEN} -t robotics-deployment:embedded -f Dockerfile .
