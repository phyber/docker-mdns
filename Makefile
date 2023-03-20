#
CROSS := cross
DOCKER := docker
ECHO := echo
PROJECT := docker-mdns

# The version of Debian used in the final generated images
DOCKER_DEBIAN_VERSION := bullseye-20230227-slim

# AARCH64
ARCH_AARCH64 := arm64
AARCH64 := aarch64
DOCKER_IMAGE_ARCH_AARCH64 := arm64/v8
PLATFORM_AARCH64 := aarch64
TARGET_AARCH64 := aarch64-unknown-linux-gnu

# AMD64
ARCH_AMD64 := amd64
AMD64 := amd64
DOCKER_IMAGE_ARCH_AMD64 := amd64
PLATFORM_AMD64 := amd64
TARGET_AMD64 := x86_64-unknown-linux-gnu

# ARMv7
ARCH_ARMV7 := armhf
ARMV7 := armv7
DOCKER_IMAGE_ARCH_ARMV7 := arm/v7
PLATFORM_ARMV7 := arm/v7
TARGET_ARMV7 := $(ARMV7)-unknown-linux-gnueabihf

target/$(TARGET_ARMV7)/release/docker-mdns:
	@$(CROSS) build \
		--release \
		--target=$(TARGET_ARMV7)

.PHONY: imagearmv7
imagearmv7: target/$(TARGET_ARMV7)/release/docker-mdns
	@$(ECHO) "Building $(PROJECT) image"

	@$(DOCKER) buildx build \
		--build-arg DEBIAN_VERSION=$(DOCKER_DEBIAN_VERSION) \
		--build-arg IMAGE_ARCH=$(DOCKER_IMAGE_ARCH_ARMV7) \
		--build-arg TARGET=$(TARGET_ARMV7) \
		--file docker/Dockerfile \
		--platform linux/$(DOCKER_IMAGE_ARCH_ARMV7) \
		--tag docker-mdns:$(ARMV7) \
		.

target/$(TARGET_AMD64)/release/docker-mdns:
	@$(CROSS) build \
		--release \
		--target=$(TARGET_AMD64)

.PHONY: imageamd64
imageamd64: target/$(TARGET_AMD64)/release/docker-mdns
	@$(ECHO) "Building $(PROJECT) image for $(PLATFORM_AMD64)"
	@$(ECHO) "Image platform: $(DOCKER_IMAGE_ARCH_AMD64)"

	@$(DOCKER) buildx build \
		--build-arg DEBIAN_VERSION=$(DOCKER_DEBIAN_VERSION) \
		--build-arg IMAGE_ARCH=$(DOCKER_IMAGE_ARCH_AMD64) \
		--build-arg TARGET=$(TARGET_AMD64) \
		--file docker/Dockerfile \
		--platform linux/$(DOCKER_IMAGE_ARCH_AMD64) \
		--tag docker-mdns:$(AMD64) \
		.

target/$(TARGET_AARCH64)/release/docker-mdns:
	@$(CROSS) build \
		--release \
		--target=$(TARGET_AARCH64)

.PHONY: imageaarch64
imageaarch64: target/$(TARGET_AARCH64)/release/docker-mdns
	@$(ECHO) "Building $(PROJECT) image for $(PLATFORM_AARCH64)"

	$(DOCKER) buildx build \
		--build-arg DEBIAN_VERSION=$(DOCKER_DEBIAN_VERSION) \
		--build-arg IMAGE_ARCH=$(DOCKER_IMAGE_ARCH_AARCH64) \
		--build-arg TARGET=$(TARGET_AARCH64) \
		--file docker/Dockerfile \
		--platform linux/$(DOCKER_IMAGE_ARCH_AARCH64) \
		--tag docker-mdns:$(AARCH64) \
		.
