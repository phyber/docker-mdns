# vim:ft=dockerfile:
ARG BASE_IMAGE
ARG IMAGE_ARCH
FROM --platform=linux/${IMAGE_ARCH} ${BASE_IMAGE}

ARG USERNAME=docker-mdns
ARG USER_ID=5000
ARG GROUP_ID=$USER_ID
RUN \
    groupadd \
        --gid $GROUP_ID \
        $USERNAME \
    && useradd \
        --create-home \
        --gid $GROUP_ID \
        --uid $USER_ID \
        $USERNAME

ARG TARGET
COPY target/${TARGET}/release/docker-mdns /usr/local/bin/

USER $USERNAME
ENTRYPOINT ["docker-mdns"]
