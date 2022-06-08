.PHONY: build
build:
	cargo build --release
	cargo build --target wasm32-wasi --release --manifest-path ./examples/kv-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/mq-sender-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/mq-receiver-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/lockd-demo/Cargo.toml
	
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
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-sender-demo.wasm -c 'mq:///tmp' &
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-receiver-demo.wasm -c 'mq:///tmp'
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-sender-demo.wasm -c 'azmq://wasi-cloud-servicebus@wasi-cloud-queue' &
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-receiver-demo.wasm -c 'azmq://wasi-cloud-servicebus@wasi-cloud-queue'
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/lockd-demo.wasm -c 'etcdlockd://localhost:2379' &
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/lockd-demo.wasm -c 'etcdlockd://localhost:2379'
