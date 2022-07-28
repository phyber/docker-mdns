DOCKER := docker
ECHO := echo
PROJECT := docker-mdns

# The version of Debian used in the final generated images
DEBIAN_VERSION := bullseye-20220711-slim

# AARCH64
ARCH_AARCH64 := arm64
AARCH64 := aarch64
PLATFORM_AARCH64 := aarch64
TAG_AARCH64 := rustcross:dbus-$(AARCH64)
TARGET_AARCH64 := aarch64-unknown-linux-gnu

# AMD64
ARCH_AMD64 := amd64
AMD64 := amd64
PLATFORM_AMD64 := amd64
TAG_AMD64 := rustcross:dbus-$(AMD64)
TARGET_AMD64 := x86_64-unknown-linux-gnu

# ARMv7
ARCH_ARMV7 := armhf
ARMV7 := armv7
PLATFORM_ARMV7 := arm/v7
TAG_ARMV7 := rustcross:dbus-$(ARMV7)
TARGET_ARMV7 := $(ARMV7)-unknown-linux-gnueabihf

# This image is created with the docker directory as context, as we don't need
# anything from the source directories at all.
crossarmv7:
	@$(ECHO) "Creating docker image: $(TAG_ARMV7)"

	@$(DOCKER) build \
		--build-arg ARCH=$(ARCH_ARMV7) \
		--build-arg TARGET=$(TARGET_ARMV7) \
		--file docker/Dockerfile.cross-$(ARMV7) \
		--tag $(TAG_ARMV7) \
		docker/

armv7: crossarmv7
	@$(ECHO) "Building $(PROJECT)"
	@$(PWD)/cross.sh $(ARMV7)

# Builds an image for running docker-mdns from.
imagearmv7: armv7
	@$(ECHO) "Building $(PROJECT) image"

	@$(DOCKER) build \
		--build-arg ARCH=$(ARCH_ARMV7) \
		--build-arg DEBIAN_VERSION=$(DEBIAN_VERSION) \
		--build-arg TARGET=$(TARGET_ARMV7) \
		--file docker/Dockerfile.$(ARMV7) \
		--tag docker-mdns:$(ARMV7) \
		.

crossamd64:
	@$(ECHO) "Creating docker image: $(TAG_AMD64)"

	@$(DOCKER) build \
		--build-arg ARCH=$(ARCH_AMD64) \
		--build-arg TARGET=$(TARGET_AMD64) \
		--file docker/Dockerfile.cross-$(AMD64) \
		--tag $(TAG_AMD64) \
		docker/

amd64: crossamd64
	@$(ECHO) "Building $(PROJECT)"
	@$(PWD)/cross.sh $(AMD64)

imageamd64: amd64
	@$(ECHO) "Building $(PROJECT) image for $(PLATFORM_AMD64)"

	@$(DOCKER) build \
		--build-arg DEBIAN_VERSION=$(DEBIAN_VERSION) \
		--build-arg PLATFORM=$(PLATFORM_AMD64) \
		--build-arg TARGET=$(TARGET_AMD64) \
		--file docker/Dockerfile.$(AMD64) \
		--tag docker-mdns:$(AMD64) \
		.

crossaarch64:
	@$(ECHO) "Creating docker image: $(TAG_AARCH64)"

	@$(DOCKER) build \
		--build-arg ARCH=$(ARCH_AARCH64) \
		--build-arg TARGET=$(TARGET_AARCH64) \
		--file docker/Dockerfile.cross-$(AARCH64) \
		--tag $(TAG_AARCH64) \
		docker/

aarch64: crossaarch64
	@$(ECHO) "Building $(PROJECT)"
	@$(PWD)/cross.sh $(AARCH64)

imageaarch64: aarch64
	@$(ECHO) "Building $(PROJECT) image for $(PLATFORM_AARCH64)"

	@$(DOCKER) build \
		--build-arg ARCH=$(ARCH_AARCH64) \
		--build-arg DEBIAN_VERSION=$(DEBIAN_VERSION) \
		--build-arg TARGET=$(TARGET_AARCH64) \
		--file docker/Dockerfile.$(AARCH64) \
		--tag docker-mdns:$(AARCH64) \
		.
