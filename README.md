# wasi-cloud
a proof of concept for a wasi-based cloud service specification

## Structure
- `/wit`: the wasi-cloud specification written in `*.wit` format (see [WIT](https://github.com/bytecodealliance/wit-bindgen/blob/main/WIT.md))
- `/src`: the wasi-cloud host cli 
- `/crates`: host implementation
- `/examples`: guest examples
- `/tests`: guest tests

## Build
- Run `make build`

## Run
- Run `./target/release/wasi-cloud -m ./target/wasm32-wasi/release/file-demo.wasm`
