FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt update && \
    apt install -y \
        clang \
        build-essential \
        git \
        curl \
        ca-certificates \
        vim && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /work

CMD ["bash"]
