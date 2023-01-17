INSTALL_DIR_PREFIX ?= /usr/local
SLIGHT ?= ./target/release/slight
LOG_LEVEL ?= slight=trace
# ^^^ To see an individual crate's trace, use LOG_LEVEL=<crate_name>=trace
# (e.g., LOG_LEVEL=runtime=trace)

### GENERAL COMMANDS
.PHONY: improve
improve:
	# --all-target: apply clippy to all targets
	# --all-features: check all available features
	# --workspace: check all packages in a workspace
	cargo clippy --all-targets --all-features --workspace -- -D warnings
	cargo fmt --all -- --check

.PHONY: build
build:
	cargo build --release --manifest-path ./slight/Cargo.toml

.PHONY: test
test:
	RUST_LOG=$(LOG_LEVEL) cargo test --release --all --no-fail-fast -- --skip integration_tests --nocapture --include-ignored

.PHONY: test-integration
test-integration:
	RUST_LOG=$(LOG_LEVEL) cargo test --test integration --no-fail-fast  -- --nocapture
### END OF GENERAL COMMANDS

### INSTALLS
.PHONY: install-deps
install-deps:
	set -x
	curl -sS -L -O https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-15/wasi-sdk-15.0-linux.tar.gz
	tar xf wasi-sdk-15.0-linux.tar.gz
	sudo mkdir -p /opt/wasi-sdk
	sudo mv wasi-sdk-15.0/* /opt/wasi-sdk/
	sudo rm -rf wasi-sdk-*
	sudo apt-get update
	sudo apt-get install lsb-release -y
	sudo apt-get install redis-server -y
	sudo apt-get install protobuf-compiler -y

.PHONY: install-deps-macos
install-deps-macos:
	set -x
	curl -sS -L -O https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-15/wasi-sdk-15.0-macos.tar.gz
	tar xf wasi-sdk-15.0-macos.tar.gz
	sudo mkdir -p /opt/wasi-sdk
	sudo mv wasi-sdk-15.0/* /opt/wasi-sdk/
	sudo rm -rf wasi-sdk-*
	chmod +x /opt/wasi-sdk/bin/clang
	brew update
	brew install protobuf
	brew install redis || true

.PHONY: install-deps-win
install-deps-win:
	choco install openssl -y
	choco install wget -y
	choco install protoc -y
	wget -O wasi-sdk-15.0-mingw.tar.gz https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-15/wasi-sdk-15.0-mingw.tar.gz
	mkdir -p C:\wasi-sdk
	tar -xvzf wasi-sdk-15.0-mingw.tar.gz --strip-components=1 -C C:\wasi-sdk
	curl -o redis-latest.zip -L https://github.com/MicrosoftArchive/redis/releases/download/win-3.0.504/Redis-x64-3.0.504.zip
	mkdir C:\redis
	tar -xvzf redis-latest.zip -C C:\redis

.PHONY: install-slight
install-slight:
	install ./target/release/slight $(INSTALL_DIR_PREFIX)/bin
### END OF INSTALLS

### RUST TESTS
.PHONY: build-rust-integration-tests
build-rust-integration-tests:
	cargo build --target wasm32-wasi --release --manifest-path ./tests/configs-test/Cargo.toml & \
	cargo build --target wasm32-wasi --release --manifest-path ./tests/keyvalue-test/Cargo.toml & \
	cargo build --target wasm32-wasi --release --manifest-path ./tests/http-test/Cargo.toml & \
	wait; \
	/bin/sh -c 'echo "DONE"'

### END OF RUST TESTS

### RUST EXAMPLES
.PHONY: build-rust
build-rust:
	cargo build --target wasm32-wasi --release --manifest-path ./examples/multi_capability-demo/Cargo.toml & \
	cargo build --target wasm32-wasi --release --manifest-path ./examples/configs-demo/Cargo.toml & \
	cargo build --target wasm32-wasi --release --manifest-path ./examples/keyvalue-demo/Cargo.toml & \
	cargo build --target wasm32-wasi --release --manifest-path ./examples/distributed-locking-demo/Cargo.toml & \
	cargo build --target wasm32-wasi --release --manifest-path ./examples/messaging-producer-demo/Cargo.toml & \
	cargo build --target wasm32-wasi --release --manifest-path ./examples/messaging-consumer-demo/Cargo.toml & \
	cargo build --target wasm32-wasi --release --manifest-path ./examples/http-server-demo/Cargo.toml & \
	cargo build --target wasm32-wasi --release --manifest-path ./examples/http-client-demo/Cargo.toml & \
	wait; \
	/bin/sh -c 'echo "DONE"'


# The dependencies to run this rule include:
# - etcd,
# - mosquitto,
# - redis, and
# - python.
.PHONY: run-rust
run-rust:
	# multi_capability
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/multi_capability-demo/slightfile.toml' run -m ./examples/multi_capability-demo/target/wasm32-wasi/release/multi_capability-demo.wasm
	# keyvalue.filesystem
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/keyvalue-demo/keyvalue_filesystem_slightfile.toml' run -m ./examples/keyvalue-demo/target/wasm32-wasi/release/keyvalue-demo.wasm
	# keyvalue.awsdynamodb
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/keyvalue-demo/keyvalue_awsdynamodb_slightfile.toml' run -m ./examples/keyvalue-demo/target/wasm32-wasi/release/keyvalue-demo.wasm
	# keyvalue.azblob
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/keyvalue-demo/keyvalue_azblob_slightfile.toml' run -m ./examples/keyvalue-demo/target/wasm32-wasi/release/keyvalue-demo.wasm
	# keyvalue.redis
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/keyvalue-demo/keyvalue_redis_slightfile.toml' run -m ./examples/keyvalue-demo/target/wasm32-wasi/release/keyvalue-demo.wasm
	# configs.usersecrets
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/configs-demo/usersecrets_slightfile.toml' run -m ./examples/configs-demo/target/wasm32-wasi/release/configs-demo.wasm
	# configs.envvars
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/configs-demo/envvars_slightfile.toml' run -m ./examples/configs-demo/target/wasm32-wasi/release/configs-demo.wasm
	# configs.azapp
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/configs-demo/azapp_slightfile.toml' run -m ./examples/configs-demo/target/wasm32-wasi/release/configs-demo.wasm
	# distributed_locking.etcd
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/distributed-locking-demo/slightfile.toml' run -m ./examples/distributed-locking-demo/target/wasm32-wasi/release/distributed-locking-demo.wasm &
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/distributed-locking-demo/slightfile.toml' run -m ./examples/distributed-locking-demo/target/wasm32-wasi/release/distributed-locking-demo.wasm
	# messaging.filesystem
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/messaging-consumer-demo/filesystem_slightfile.toml' run -m ./examples/messaging-consumer-demo/target/wasm32-wasi/release/messaging-consumer-demo.wasm &
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/messaging-producer-demo/filesystem_slightfile.toml' run -m ./examples/messaging-producer-demo/target/wasm32-wasi/release/messaging-producer-demo.wasm
	# messaging.azsbus
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/messaging-consumer-demo/azsbus_slightfile.toml' run -m ./examples/messaging-consumer-demo/target/wasm32-wasi/release/messaging-consumer-demo.wasm &
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/messaging-producer-demo/azsbus_slightfile.toml' run -m ./examples/messaging-producer-demo/target/wasm32-wasi/release/messaging-producer-demo.wasm
	# messaging.mosquitto
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/messaging-consumer-demo/mosquitto_slightfile.toml' run -m ./examples/messaging-consumer-demo/target/wasm32-wasi/release/messaging-consumer-demo.wasm &
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/messaging-producer-demo/mosquitto_slightfile.toml' run -m ./examples/messaging-producer-demo/target/wasm32-wasi/release/messaging-producer-demo.wasm
	# messaging.confluent_apache_kafka
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/messaging-consumer-demo/caf_slightfile.toml' run -m ./examples/messaging-consumer-demo/target/wasm32-wasi/release/messaging-consumer-demo.wasm &
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/messaging-producer-demo/caf_slightfile.toml' run -m ./examples/messaging-producer-demo/target/wasm32-wasi/release/messaging-producer-demo.wasm

.PHONY: clean-rust
clean-rust:
	cargo clean --manifest-path ./examples/configs-demo/Cargo.toml & \
	cargo clean --manifest-path ./examples/multi_capability-demo/Cargo.toml & \
	cargo clean --manifest-path ./examples/keyvalue-demo/Cargo.toml & \
	cargo clean --manifest-path ./examples/distributed-locking-demo/Cargo.toml & \
	cargo clean --manifest-path ./examples/messaging-producer-demo/Cargo.toml & \
	cargo clean --manifest-path ./examples/messaging-consumer-demo/Cargo.toml & \
	cargo clean --manifest-path ./examples/http-client-demo/Cargo.toml & \
	cargo clean --manifest-path ./examples/http-server-demo/Cargo.toml & \
	wait; \
	/bin/sh -c 'echo "DONE"'

### END OF RUST EXAMPLES

### C EXAMPLES

# To run this rule, you'll need wit-bindgen v0.2.0 installed (cargo install --git https://github.com/bytecodealliance/wit-bindgen wit-bindgen-cli --tag v0.2.0)
# .PHONY: build-c
build-c:
	$(MAKE) -C examples/multi_capability-demo-clang/ clean
	$(MAKE) -C examples/multi_capability-demo-clang/ bindings
	$(MAKE) -C examples/multi_capability-demo-clang/ build

.PHONY: build-c-win
build-c-win:
	$(MAKE) -C examples/multi_capability-demo-clang/ clean
	$(MAKE) -C examples/multi_capability-demo-clang/ bindings
	$(MAKE) -C examples/multi_capability-demo-clang/ build-win


.PHONY: run-c
run-c:
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/messaging-producer-demo/azsbus_slightfile.toml' run -m ./examples/messaging-producer-demo/target/wasm32-wasi/release/messaging-producer-demo.wasm && $(SLIGHT) -c './examples/multi_capability-demo-clang/slightfile.toml' run -m ./examples/multi_capability-demo-clang/multi_capability-demo-clang.wasm
### END OF C EXAMPLES

### APP DEMO
.PHONY: build-app-demos
build-app-demos:
	cargo build --target wasm32-wasi --release --manifest-path ./examples/app-demos/restaurant-backend/Cargo.toml

.PHONY: run-restaurant-backend
run-restaurant-backend:
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c ./examples/app-demos/restaurant-backend/slightfile.toml run -m ./examples/app-demos/restaurant-backend/target/wasm32-wasi/release/restaurant-backend.wasm
	
### END OF APP DEMO

### GITHUB RELEASES
.PHONY: prepare-release
prepare-release:
	tar -C target/ -czvf slight-linux-x86_64.tar.gz release/slight
	tar -C templates/ -czvf rust-template.tar.gz rust
	tar -C templates/ -czvf c-template.tar.gz c

.PHONY: prepare-release-win
prepare-release-win:
	tar -C target/ -czvf slight-windows-x86_64.tar.gz release/slight.exe

.PHONY: prepare-release-mac
prepare-release-mac:
	tar -C target/ -czvf slight-macos-amd64.tar.gz release/slight	

### END OF GITHUB RELEASES