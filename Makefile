INSTALL_DIR_PREFIX ?= /usr/local
SLIGHT ?= ./target/release/slight


.PHONY: build
build:
	cargo build --release
	cargo build --release --manifest-path ./slight/Cargo.toml
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

.PHONY: check
check:
	cargo clippy --all-targets --all-features -- -D warnings
	cargo fmt --all -- --check

.PHONY: run
run:
	### running multi capability example
	$(SLIGHT) -c './examples/multi_capability-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/multi_capability-demo.wasm
	### running watch example
	$(SLIGHT) -c './examples/kv-demo/filekv.toml' run -m ./target/wasm32-wasi/release/kv-demo.wasm & python ./examples/kv-demo/simulate.py
	### running azblobkv example
	# $(SLIGHT) -c './examples/kv-demo/azblobkv.toml' run -m ./target/wasm32-wasi/release/kv-demo.wasm	
	### running filemq example
	$(SLIGHT) -c './examples/mq-sender-demo/filemq.toml' run -m ./target/wasm32-wasi/release/mq-sender-demo.wasm &
	$(SLIGHT) -c './examples/mq-receiver-demo/filemq.toml' run -m ./target/wasm32-wasi/release/mq-receiver-demo.wasm
	### running azsbusmq example
	$(SLIGHT) -c './examples/mq-sender-demo/azsbusmq.toml' run -m ./target/wasm32-wasi/release/mq-sender-demo.wasm &
	$(SLIGHT) -c './examples/mq-receiver-demo/azsbusmq.toml' run -m ./target/wasm32-wasi/release/mq-receiver-demo.wasm
	### running etcdlockd example
	$(SLIGHT) -c './examples/lockd-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/lockd-demo.wasm &
	$(SLIGHT) -c './examples/lockd-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/lockd-demo.wasm
	### running ckpubsub example
	$(SLIGHT) -c './examples/pubsub-consumer-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/pubsub-consumer-demo.wasm &
	$(SLIGHT) -c './examples/pubsub-producer-demo/slightfile.toml' run -m ./target/wasm32-wasi/release/pubsub-producer-demo.wasm

run-c:
	### running c example
	$(SLIGHT) -c './examples/mq-sender-demo/filemq.toml' run -m ./target/wasm32-wasi/release/mq-sender-demo.wasm && $(SLIGHT) -c './examples/kv-mq-demo-clang/slightfile.toml' run -m ./examples/kv-mq-demo-clang/kv-mq-filesystem-c.wasm

install:
	install ./target/release/slight $(INSTALL_DIR_PREFIX)/bin

feature-run:
	$(SLIGHT) -c './examples/kv-demo/azblobkv-wc.toml' run -m ./target/wasm32-wasi/release/kv-demo.wasm
