WASI_VERSION = 15
WASI_VERSION_FULL = $(WASI_VERSION).0
WASI_SDK_PATH ?= /opt/wasi-sdk

WASI_SDK ?= $(WASI_SDK_PATH)
WASI_CC = $(WASI_SDK)/bin/clang
DEBUG_OPT = -glldb
SYSROOT = $(WASI_SDK)/share/wasi-sysroot/

TARGET ?= {{project-name}}.wasm
SRC_DIR ?= ./src

.PHONY: install-deps
install-deps:
	set -x
	curl -sS -L -O https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-$(WASI_VERSION)/wasi-sdk-$(WASI_VERSION_FULL)-linux.tar.gz
	tar xf wasi-sdk-$(WASI_VERSION_FULL)-linux.tar.gz
	sudo rm -rf /opt/wasi-sdk
	sudo mkdir -p /opt/wasi-sdk
	sudo mv wasi-sdk-$(WASI_VERSION_FULL)/* /opt/wasi-sdk/
	sudo rm -rf wasi-sdk-*

.PHONY: clean
clean:
	rm -rf bindings/
	mkdir bindings/

.PHONY: bindings
bindings:
	wit-bindgen c --import ../../wit/kv.wit --out-dir bindings/
	wit-bindgen c --import ../../wit/mq.wit --out-dir bindings/
	chmod +x bindings/kv.c bindings/mq.c

.PHONY: build
build:
	sudo $(WASI_CC) -I . -I ./bindings -c -o kv.o bindings/kv.c
	sudo $(WASI_CC) -I . -I ./bindings -c -o mq.o bindings/mq.c
	sudo $(WASI_CC) -Wall $(SRC_DIR)/main.c kv.o mq.o -o $(TARGET) --sysroot $(SYSROOT)
# ^^^ note: added sudo to make sure that `clang` sees kv.c, etc as executable files

.PHONY: build-win
build-win:
	$(WASI_SDK)/bin/clang.exe -I . -I ./bindings -c -o kv.o bindings/kv.c
	$(WASI_SDK)/bin/clang.exe -I . -I ./bindings -c -o mq.o bindings/mq.c
	$(WASI_SDK)/bin/clang.exe -Wall $(SRC_DIR)/main.c kv.o mq.o -o $(TARGET) --sysroot $(SYSROOT)