INSTALL_DIR_PREFIX ?= /usr/local
SLIGHT ?= ./target/release/slight
LOG_LEVEL ?= slight=trace
# ^^^ To see an individual crate's trace, use LOG_LEVEL=<crate_name>=trace
# (e.g., LOG_LEVEL=runtime=trace)

.PHONY: build
build:
	cargo build --release
	cargo build --release --manifest-path ./slight/Cargo.toml

.PHONY: test
test:
	cargo test --all --no-fail-fast -- --nocapture

.PHONY: improve
improve:
	cargo clippy --all-targets --all-features -- -D warnings
	cargo fmt --all -- --check

.PHONY: install
install:
	install ./target/release/slight $(INSTALL_DIR_PREFIX)/bin

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
