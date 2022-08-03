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
	cargo build --release
	cargo build --release --manifest-path ./slight/Cargo.toml

.PHONY: test
test:
	RUST_LOG=$(LOG_LEVEL) cargo test --all --no-fail-fast -- --skip integration_tests --nocapture --include-ignored

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

.PHONY: install-slight
install-slight:
	install ./target/release/slight $(INSTALL_DIR_PREFIX)/bin
### END OF INSTALLS

### RUST EXAMPLES
.PHONY: build-rust
build-rust:
	cargo build --target wasm32-wasi --release --manifest-path ./examples/configs-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/watch-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/multi_capability-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/kv-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/mq-sender-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/mq-receiver-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/lockd-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/pubsub-producer-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/pubsub-consumer-demo/Cargo.toml

.PHONY: run-rust
run-rust:
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/multi_capability-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/multi_capability-demo.wasm
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/kv-demo/kvfilesystem_slightfile.toml' run -m ./target/wasm32-wasi/release/kv-demo.wasm
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/kv-demo/kvazblob_slightfile.toml' run -m ./target/wasm32-wasi/release/kv-demo.wasm	
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/watch-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/watch-demo.wasm & 
	python ./examples/watch-demo/simulate.py
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/configs-demo/usersecrets_slightfile.toml' run -m ./target/wasm32-wasi/release/configs-demo.wasm
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/configs-demo/envvars_slightfile.toml' run -m ./target/wasm32-wasi/release/configs-demo.wasm
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/mq-sender-demo/mqfilesystem_slightfile.toml' run -m ./target/wasm32-wasi/release/mq-sender-demo.wasm &
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/mq-receiver-demo/mqfilesystem_slightfile.toml' run -m ./target/wasm32-wasi/release/mq-receiver-demo.wasm
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/mq-sender-demo/mqazsbus_slightfile.toml' run -m ./target/wasm32-wasi/release/mq-sender-demo.wasm &
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/mq-receiver-demo/mqazsbus_slightfile.toml' run -m ./target/wasm32-wasi/release/mq-receiver-demo.wasm
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/lockd-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/lockd-demo.wasm &
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/lockd-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/lockd-demo.wasm
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/pubsub-consumer-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/pubsub-consumer-demo.wasm &
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/pubsub-producer-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/pubsub-producer-demo.wasm
### END OF RUST EXAMPLES

### C EXAMPLES
.PHONY: build-c
build-c:
	$(MAKE) -C examples/multi_capability-demo-clang/ clean
	$(MAKE) -C examples/multi_capability-demo-clang/ bindings
	$(MAKE) -C examples/multi_capability-demo-clang/ build

.PHONY: run-c
run-c:
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/mq-sender-demo/mqfilesystem_slightfile.toml' run -m ./target/wasm32-wasi/release/mq-sender-demo.wasm && $(SLIGHT) -c './examples/multi_capability-demo-clang/slightfile.toml' run -m ./examples/multi_capability-demo-clang/multi_capability-demo-clang.wasm
### END OF C EXAMPLES
