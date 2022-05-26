.PHONY: build
build:
	cargo build --release
	cargo build --target wasm32-wasi --release --manifest-path ./examples/kv-filesystem-config/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/kv-azure-blob-config/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/kv-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/mq-filesystem-config/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/mq-azure-servicebus-config/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/mq-sender-demo/Cargo.toml
	cargo build --target wasm32-wasi --release --manifest-path ./examples/mq-receiver-demo/Cargo.toml

.PHONY: test
test:
	# cargo test --all --no-fail-fast -- --nocapture
	cargo clippy --all-targets --all-features -- -D warnings
	cargo fmt --all -- --check

.PHONY: run
run:
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/kv-demo.wasm -c ./target/wasm32-wasi/release/kv_filesystem_config.wasm
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/kv-demo.wasm -c ./target/wasm32-wasi/release/kv_azure_blob_config.wasm

run-mq-filesystem-sender:
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-sender-demo.wasm -c ./target/wasm32-wasi/release/mq_filesystem_config.wasm

run-mq-filesystem-receiver:
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-receiver-demo.wasm -c ./target/wasm32-wasi/release/mq_filesystem_config.wasm

run-mq-azure-servicebus-sender:
	./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-sender-demo.wasm -c ./target/wasm32-wasi/release/mq_azure_servicebus_config.wasm

run-mq-azure-servicebus-receiver:
	#./target/release/wasi-cloud -m ./target/wasm32-wasi/release/mq-receiver-demo.wasm -c ./target/wasm32-wasi/release/mq_azure_servicebus_config.wasm
	python3 ./examples/mq-receiver-demo/src/main.py