# Directories
OUT = out
ISO = iso
ARCH ?= x86_64
RELEASE ?= debug
BOOT = src/arch/$(ARCH)/boot

# Input files
LINK_LD = $(BOOT)/link.ld
GRUB_CFG = $(ISO)/boot/grub/grub.cfg

# Files we make
S_FILES = $(shell find $(BOOT) -type f -name '*.S' | sort)
O_FILES = $(S_FILES:$(BOOT)/%.S=$(OUT)/%.o)
BOOT_BIN = $(ISO)/boot/boot.bin
TARGET_DIR = target/$(ARCH)-interkern/$(RELEASE)
INTERKERN_A = $(TARGET_DIR)/libinterkern_rs.a
KERN_ISO = $(OUT)/interkern.iso

ifeq ($(RELEASE),release)
	CARGO_FLAGS = --release
endif

.PHONY: clean iso release rs

iso: $(KERN_ISO)

release:
	make RELEASE=release

$(KERN_ISO): $(BOOT_BIN) $(GRUB_CFG)
	grub-mkrescue -o $(KERN_ISO) iso

$(BOOT_BIN): $(O_FILES) $(LINK_LD) rs
	ld -n --gc-sections -T $(LINK_LD) $(O_FILES) $(INTERKERN_A) -o $(BOOT_BIN)
	objcopy --only-keep-debug $@ $@.debug
	strip -g $@

$(OUT)/%.o: $(BOOT)/%.S | $(OUT)
	as -g $< -o $@

$(OUT):
	mkdir -p $(OUT)

rs:
	RUST_TARGET_PATH=$(PWD)/targets xargo build --target $(ARCH)-interkern $(CARGO_FLAGS)

clean:
	rm -rf $(OUT) $(BOOT_BIN)
	xargo clean

