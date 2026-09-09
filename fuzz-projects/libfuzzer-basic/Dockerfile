FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive
ENV ASAN_SYMBOLIZER_PATH=/usr/bin/llvm-symbolizer

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        clang \
        libclang-rt-dev \
        llvm && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /work

ENTRYPOINT ["bash", "/work/fuzz-projects/libfuzzer-basic/run_fuzzer.sh"]
