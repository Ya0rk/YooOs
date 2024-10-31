# Building variables
PACKAGE_NAME = YooOs
BOOTLOADER = default
TARGET = riscv64gc-unknown-none-elf
export BOARD = qemu
export MODE = debug

# Tools
QEMU = qemu-system-riscv64
GDB = riscv64-elf-gdb
OBJDUMP = rust-objdump --arch-name=riscv64
OBJCOPY = rust-objcopy --binary-architecture=riscv64
PAGER ?= less

# Args
DISASM_ARGS = -d
QEMU_ARGS = -machine virt \
			 -nographic \
			 -bios $(BOOTLOADER) \
			 -kernel $(KERNEL_ELF)

# Target files
TARGET_DIR := os/target/$(TARGET)/$(MODE)
KERNEL_ELF := $(TARGET_DIR)/$(PACKAGE_NAME)
KERNEL_ASM := $(TARGET_DIR)/$(PACKAGE_NAME).asm

# Default target
.PHONY: all
all: $(KERNEL_ELF) $(KERNEL_ASM)

# Build target
.PHONY: build
build: 
	@echo "Building for platform: $(BOARD)"
	@cd os && cargo build
	@echo "Updated: $(KERNEL_ELF)"

# Assembling target
$(KERNEL_ASM): $(KERNEL_ELF)
	@$(OBJDUMP) $(DISASM_ARGS) $(KERNEL_ELF) > $(KERNEL_ASM)
	@echo "Updated: $(KERNEL_ASM)"

# Run target
.PHONY: run
run: build
	@$(QEMU) $(QEMU_ARGS)

# Clean target
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	@cd os && cargo clean
	@rm -rf $(TARGET_DIR)/*
	@echo "Cleaned: $(TARGET_DIR) and related files."

# Disassembly target
.PHONY: disasm
disasm: $(KERNEL_ASM)
	@cat $(KERNEL_ASM) | $(PAGER)

# GDB server target
.PHONY: gdbserver
gdbserver: build
	@$(QEMU) $(QEMU_ARGS) -s -S

# GDB client target
.PHONY: gdbclient
gdbclient: build
	@$(GDB) -ex 'file $(KERNEL_ELF)' \
			-ex 'set arch riscv:rv64' \
			-ex 'target remote localhost:1234'

# Mark all phony targets
.PHONY: all build run clean disasm gdbserver gdbclient
