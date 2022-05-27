.PHONY: build
build:
	cargo build --release
	
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
	# ./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-filesystem-sender-demo.wasm -c 'mq:///tmp'
	# ./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-filesystem-receiver-demo.wasm -c 'mq:///tmp'
