.PHONY: build
build:
	cargo build --release
	install ./target/release/wasi-cloud /usr/local/bin
	cargo build --target wasm32-wasi --release --manifest-path ./examples/kv-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/mq-filesystem-sender-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/mq-filesystem-receiver-demo/Cargo.toml

.PHONY: test
test:
	# cargo test --all --no-fail-fast -- --nocapture
	cargo clippy --all-targets --all-features -- -D warnings
	cargo fmt --all -- --check

.PHONY: run
run:
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/kv-demo.wasm -c 'file:///tmp'
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/kv-demo.wasm -c 'azblob://my-container'
	# ./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-filesystem-sender-demo.wasm -c 'mq:///tmp'
	# ./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-filesystem-receiver-demo.wasm -c 'mq:///tmp'
