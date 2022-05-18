.PHONY: build
build:
	cargo build --release
	cargo build --target wasm32-wasi --release --manifest-path ./examples/mq-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/kv-filesystem-demo/Cargo.toml

.PHONY: test
test:
	# cargo test --all --no-fail-fast -- --nocapture
	cargo clippy --all-targets --all-features -- -D warnings
	cargo fmt --all -- --check

run-kv:
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/kv-filesystem-demo.wasm

run-mq:
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-demo.wasm