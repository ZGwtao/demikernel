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
ifneq ($(DEMKI_APP),udp-echo)
$(error Unsupported demikernel app target for uk-on-mk: $(DEMKI_APP))
endif

RUST_TOOLCHAIN ?= nightly-2026-08-21
CARGO ?= cargo
APP_ELF := demikernel.elf

APP_SOURCE := $(ROOT)/dep/demikernel/examples/rust/$(DEMKI_APP).rs

RUST_SOURCES := $(shell find $(ROOT)/dep/demikernel/src $(ROOT)/sddf-rs/src \
    -type f \( -name '*.rs' -o -name '*.c' -o -name '*.h' \))

APP_DEPS := \
    $(APP_SOURCE) \
    $(EXAMPLE_DIR)/src/main.rs \
    $(EXAMPLE_DIR)/Cargo.toml \
    $(ROOT)/sddf-rs/build.rs \
    $(RUST_TARGET_JSON) \
    $(RUST_SOURCES)

$(APP_ELF): $(APP_DEPS) | $(BUILD_DIR)
	RUSTUP_TOOLCHAIN=$(RUST_TOOLCHAIN) RUSTC_BOOTSTRAP=1 \
		$(CARGO) build \
			--manifest-path $(EXAMPLE_DIR)/Cargo.toml \
			-Z json-target-spec \
			-Z build-std=core,alloc,compiler_builtins \
			-Z build-std-features=compiler-builtins-mem \
			--target $(RUST_TARGET_JSON) \
			--target-dir $(BUILD_DIR)/cargo \
			--release
	cp $(BUILD_DIR)/cargo/$(RUST_TARGET)/release/demikernel.elf $@
