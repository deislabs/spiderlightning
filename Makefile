INSTALL_DIR_PREFIX ?= /usr/local
SLIGHT ?= ./target/release/slight

# If you want to see individual crate's trace, please use LOG_LEVEL=<crate_name>=trace
# Example: LOG_LEVEL=runtime=trace
LOG_LEVEL ?= slight=trace

.PHONY: build
build:
	cargo build --release
	cargo build --release --manifest-path ./slight/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/configs-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/watch-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/multi_capability-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/kv-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/mq-sender-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/mq-receiver-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/lockd-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/pubsub-producer-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/pubsub-consumer-demo/Cargo.toml
	
.PHONY: test
test:
	cargo test --all --no-fail-fast -- --nocapture

.PHONY: improve
improve:
	cargo clippy --all-targets --all-features -- -D warnings
	cargo fmt --all -- --check

.PHONY: run
run:
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/multi_capability-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/multi_capability-demo.wasm
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/kv-demo/filekv.toml' run -m ./target/wasm32-wasi/release/kv-demo.wasm
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/kv-demo/azblobkv.toml' run -m ./target/wasm32-wasi/release/kv-demo.wasm	
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/watch-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/watch-demo.wasm & 
	python ./examples/watch-demo/simulate.py
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/configs-demo/usersecrets_configs.toml' run -m ./target/wasm32-wasi/release/configs-demo.wasm
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/configs-demo/envvars_configs.toml' run -m ./target/wasm32-wasi/release/configs-demo.wasm
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/mq-sender-demo/filemq.toml' run -m ./target/wasm32-wasi/release/mq-sender-demo.wasm &
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/mq-receiver-demo/filemq.toml' run -m ./target/wasm32-wasi/release/mq-receiver-demo.wasm
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/mq-sender-demo/azsbusmq.toml' run -m ./target/wasm32-wasi/release/mq-sender-demo.wasm &
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/mq-receiver-demo/azsbusmq.toml' run -m ./target/wasm32-wasi/release/mq-receiver-demo.wasm
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/lockd-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/lockd-demo.wasm &
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/lockd-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/lockd-demo.wasm
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/pubsub-consumer-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/pubsub-consumer-demo.wasm &
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/pubsub-producer-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/pubsub-producer-demo.wasm

.PHONY: build-c
build-c:
	$(MAKE) -C examples/multi_capability-demo-clang/ clean
	$(MAKE) -C examples/multi_capability-demo-clang/ bindings
	$(MAKE) -C examples/multi_capability-demo-clang/ build

.PHONY: run-c
run-c:
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/mq-sender-demo/filemq.toml' run -m ./target/wasm32-wasi/release/mq-sender-demo.wasm && $(SLIGHT) -c './examples/multi_capability-demo-clang/slightfile.toml' run -m ./examples/multi_capability-demo-clang/multi_capability-demo-clang.wasm

.PHONY: install
install:
	install ./target/release/slight $(INSTALL_DIR_PREFIX)/bin
