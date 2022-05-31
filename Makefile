.PHONY: build
build:
	cargo build --release
	cargo build --target wasm32-wasi --release --manifest-path ./examples/kv-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/mq-sender-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/mq-receiver-demo/Cargo.toml
	
.PHONY: test
test:
	cargo test --all --no-fail-fast -- --nocapture

.PHONY: check
check:
	cargo clippy --all-targets --all-features -- -D warnings
	cargo fmt --all -- --check

.PHONY: run
run:
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/kv-demo.wasm -c 'file:///tmp'
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/kv-demo.wasm -c 'azblob://my-container'

run-mq-filesystem-sender:
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-sender-demo.wasm -c 'mq:///tmp'

run-mq-filesystem-receiver:
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-receiver-demo.wasm -c 'mq:///tmp'

run-mq-azure-servicebus-sender:
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-sender-demo.wasm -c 'azmq://wasi-cloud-servicebus@wasi-cloud-queue'

run-mq-azure-servicebus-receiver:
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-receiver-demo.wasm -c 'azmq://wasi-cloud-servicebus@wasi-cloud-queue'
	
