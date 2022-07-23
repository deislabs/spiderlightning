# kv-mq-demo-clang

To run this example, you'll need to install the WASI SDK in the `./examples/kv-mq-demo-clang` folder. For linux, to do that, run:
```sh
export WASI_VERSION=14
export WASI_VERSION_FULL=${WASI_VERSION}.0
wget https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${WASI_VERSION}/wasi-sdk-${WASI_VERSION_FULL}-linux.tar.gz
tar xvf wasi-sdk-${WASI_VERSION_FULL}-linux.tar.gz
```

After that, inside the `./examples/kv-mq-demo-clang`, run `make build`.

Next, `cd` back to the root of the `spiderlightning` repo (i.e., `cd ../..`), and `make run-c`.