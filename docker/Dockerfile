# vim:ft=dockerfile:
ARG DEBIAN_VERSION
ARG IMAGE_ARCH
FROM --platform=linux/${IMAGE_ARCH} debian:${DEBIAN_VERSION}

ARG TARGET
COPY target/${TARGET}/release/docker-mdns /usr/local/bin/

ENTRYPOINT ["docker-mdns"]
