FROM ubuntu:24.04

RUN apt-get update \
    && apt-get install -y --no-install-recommends can-utils \
    && rm -rf /var/lib/apt/lists/*

COPY test_uds.sh /test_uds.sh
RUN chmod +x /test_uds.sh

ENTRYPOINT ["/test_uds.sh"]
