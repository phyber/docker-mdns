# vim:ft=dockerfile:
ARG BASE_IMAGE
ARG IMAGE_ARCH
FROM --platform=linux/${IMAGE_ARCH} ${BASE_IMAGE}

ARG TARGET
COPY target/${TARGET}/release/docker-mdns /usr/local/bin/

ENTRYPOINT ["docker-mdns"]
