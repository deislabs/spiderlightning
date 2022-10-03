WASI_VERSION = 15
WASI_VERSION_FULL = $(WASI_VERSION).0
WASI_SDK_PATH ?= /opt/wasi-sdk

WASI_SDK ?= $(WASI_SDK_PATH)
WASI_CC = $(WASI_SDK)/bin/clang
DEBUG_OPT = -glldb
SYSROOT = $(WASI_SDK)/share/wasi-sysroot/

TARGET ?= {{project-name}}.wasm
SRC_DIR ?= ./src

# install wasi-sdk used for the unix build
.PHONY: install-deps
install-deps:
	set -x
	curl -sS -L -O https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-$(WASI_VERSION)/wasi-sdk-$(WASI_VERSION_FULL)-linux.tar.gz
	tar xf wasi-sdk-$(WASI_VERSION_FULL)-linux.tar.gz
	sudo rm -rf /opt/wasi-sdk
	sudo mkdir -p /opt/wasi-sdk
	sudo mv wasi-sdk-$(WASI_VERSION_FULL)/* /opt/wasi-sdk/
	sudo rm -rf wasi-sdk-*

# install wasi-sdk used for the windows build
.PHONY: install-deps-win
install-deps-win:
	wget -O wasi-sdk-15.0-mingw.tar.gz https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-15/wasi-sdk-15.0-mingw.tar.gz
	tar -xvzf wasi-sdk-15.0-mingw.tar.gz
	mkdir -p /opt/wasi-sdk
	mv wasi-sdk-15.0/* /opt/wasi-sdk/

# clean up bindings dir for unix
.PHONY: clean
clean:
	rm -rf $(SRC_DIR)/bindings/
	mkdir $(SRC_DIR)/bindings/

# clean up bindings dir for windows
.PHONY: clean-win
clean-win:
	rm -r -Forced $(SRC_DIR)/bindings/
	mkdir $(SRC_DIR)/bindings/	

# generate new bindings
.PHONY: bindings
bindings:
	wit-bindgen c --import wit/kv_{{release}}/kv.wit --out-dir $(SRC_DIR)/bindings/

# build the project on unix
.PHONY: build
build:
	sudo $(WASI_CC) -I . -I $(SRC_DIR)/bindings -c -o kv.o $(SRC_DIR)/bindings/kv.c
	sudo $(WASI_CC) -Wall $(SRC_DIR)/main.c kv.o -o $(TARGET) --sysroot $(SYSROOT)

# build the project on windows
.PHONY: build-win
build-win:
	$(WASI_SDK)/bin/clang.exe -I . -I $(SRC_DIR)/bindings -c -o kv.o $(SRC_DIR)/bindings/kv.c
	$(WASI_SDK)/bin/clang.exe -Wall $(SRC_DIR)/main.c kv.o -o $(TARGET) --sysroot $(SYSROOT)