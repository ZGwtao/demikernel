# SPDX-License-Identifier: BSD-2-Clause

ifeq ($(strip $(SDDF)),)
$(error SDDF must be specified)
endif

ifeq ($(strip $(MICROKIT_SDK)),)
$(error MICROKIT_SDK must be specified)
endif

ifeq ($(strip $(MICROKIT_BOARD)),)
$(error MICROKIT_BOARD must be specified)
endif

ifeq ($(strip $(MICROKIT_CONFIG)),)
$(error MICROKIT_CONFIG must be specified)
endif

SEL4_INCLUDE_DIRS := $(MICROKIT_SDK)/board/$(MICROKIT_BOARD)/$(MICROKIT_CONFIG)/include
export SDDF MICROKIT_SDK MICROKIT_BOARD MICROKIT_CONFIG SEL4_INCLUDE_DIRS

DEMKI_APP ?= udp-echo
SUPPORTED_APPS := udp-echo udp-dump udp-ping-pong udp-pktgen udp-push-pop udp-relay
ifeq ($(filter $(DEMKI_APP),$(SUPPORTED_APPS)),)
$(error Unsupported demikernel app target for uk-on-mk: $(DEMKI_APP))
endif

DEMKI_MODE ?= server
ifeq ($(DEMKI_MODE),client)
ifeq ($(filter $(DEMKI_APP),udp-ping-pong udp-push-pop),)
$(error Client mode is unsupported for $(DEMKI_APP))
endif
else ifneq ($(DEMKI_MODE),server)
$(error Unsupported Demikernel mode: $(DEMKI_MODE))
endif

RUST_TOOLCHAIN ?= nightly-2026-08-21
CARGO ?= cargo
APP_ELF := demikernel.elf

APP_SOURCE := $(ROOT)/dep/demikernel/examples/rust/$(DEMKI_APP).rs

# Cargo selects the apps with app-*; mode-client switches the two
# client/server examples to client mode. Server mode is the default.
APP_FEATURES := app-$(DEMKI_APP)
ifeq ($(DEMKI_MODE),client)
APP_FEATURES += mode-client
endif

RUST_SOURCES := $(shell find $(ROOT)/dep/demikernel/src $(ROOT)/sddf-rs/src \
    -type f \( -name '*.rs' -o -name '*.c' -o -name '*.h' \))

APP_DEPS := \
    $(APP_SOURCE) \
    $(EXAMPLE_DIR)/src/main.rs \
    $(EXAMPLE_DIR)/build.rs \
    $(EXAMPLE_DIR)/Cargo.toml \
    $(ROOT)/sddf-rs/build.rs \
    $(RUST_TARGET_JSON) \
    $(RUST_SOURCES)

$(BUILD_DIR):
	mkdir -p $@

$(APP_ELF): $(APP_DEPS) | $(BUILD_DIR)
	RUSTUP_TOOLCHAIN=$(RUST_TOOLCHAIN) RUSTC_BOOTSTRAP=1 \
		$(CARGO) build \
			--manifest-path $(EXAMPLE_DIR)/Cargo.toml \
			--features "$(APP_FEATURES)" \
			-Z json-target-spec \
			-Z build-std=core,alloc,compiler_builtins \
			-Z build-std-features=compiler-builtins-mem \
			--target $(RUST_TARGET_JSON) \
			--target-dir $(BUILD_DIR)/cargo \
			--release
	cp $(BUILD_DIR)/cargo/$(RUST_TARGET)/release/demikernel.elf $@
