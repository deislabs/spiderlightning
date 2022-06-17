WASI_VERSION = 14
WASI_VERSION_FULL = $(WASI_VERSION).0
WASI_SDK_PATH ?= `pwd`/wasi-sdk-${WASI_VERSION_FULL}

WASI_SDK ?= $(WASI_SDK_PATH)
WASI_CC = $(WASI_SDK)/bin/clang
DEBUG_OPT = -glldb
SYSROOT = $(WASI_SDK)/share/wasi-sysroot/

TARGET ?= kv-filesystem-c.wasm
SRC_DIR ?= ./src

build:
	$(WASI_CC) -Wall -I . -I ./bindings -c -o kv.o bindings/kv.c
	$(WASI_CC) -Wall -I . -I ./bindings -c -o mq.o bindings/mq.c
	$(WASI_CC) -Wall  main.c kv.o mq.o -o kv-mq-filesystem-c.wasm --sysroot $(SYSROOT)