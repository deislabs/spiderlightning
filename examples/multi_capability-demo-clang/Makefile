WASI_VERSION = 15
WASI_VERSION_FULL = $(WASI_VERSION).0

WASI_SDK ?= /opt/wasi-sdk
WASI_SDK_WIN ?= C:\wasi-sdk
WASI_CC = $(WASI_SDK)/bin/clang
WASI_CC_WIN = $(WASI_SDK_WIN)\bin\clang.exe
DEBUG_OPT = -glldb
SYSROOT = $(WASI_SDK)/share/wasi-sysroot
SYSROOT_WIN = $(WASI_SDK_WIN)\share\wasi-sysroot

TARGET ?= multi_capability-demo-clang.wasm
SRC_DIR ?= ./src

clean:
	rm -rf bindings/
	mkdir bindings/

.PHONY: bindings
bindings:
	wit-bindgen c --import ../../wit/keyvalue.wit --out-dir bindings/
	wit-bindgen c --import ../../wit/messaging.wit --out-dir bindings/
	chmod +x bindings/keyvalue.c bindings/messaging.c

.PHONY: build
build:
	sudo $(WASI_CC) -I . -I ./bindings -c -o keyvalue.o bindings/keyvalue.c
	sudo $(WASI_CC) -I . -I ./bindings -c -o messaging.o bindings/messaging.c
	sudo $(WASI_CC) -Wall main.c keyvalue.o messaging.o -o $(TARGET) --sysroot $(SYSROOT)
# ^^^ note: added sudo to make sure that `clang` sees keyvalue.c, etc as executable files

.PHONY: build-win
build-win:
	$(WASI_CC_WIN) -I . -I ./bindings -c -o keyvalue.o bindings/keyvalue.c
	$(WASI_CC_WIN) -I . -I ./bindings -c -o messaging.o bindings/messaging.c
	$(WASI_CC_WIN) -Wall main.c keyvalue.o messaging.o -o $(TARGET) --sysroot $(SYSROOT_WIN)