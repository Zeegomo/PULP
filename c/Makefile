CORES ?= 8
rust_lib:
	cargo clean
	CORES=${CORES} cargo +nightly build --manifest-path ../cipher-suite/Cargo.toml --target riscv32imc-unknown-none-elf --no-default-features
	cd ../target/riscv32imc-unknown-none-elf/debug && ar x libcipher_suite.a

# COMPILER FLAGS
APP_CFLAGS += -O3 -DNUM_CORES=${CORES}

PMSIS_OS = freertos

APP = main
APP_SRCS	+= main.c 
APP_LDFLAGS = ../target/riscv32imc-unknown-none-elf/debug/*.o 

CONFIG_HYPERRAM = 1

include $(RULES_DIR)/pmsis_rules.mk
