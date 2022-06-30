<img align="right" src="assets/tmp-logo.png" width="300px" />

# WASI-Cloud
WASI-Cloud defines a set of WebAssembly Interface Types (i.e., WIT) files that abstract the cloud-provider specific knowledge required behind utilizing a Cloud serivce (e.g., key-value store, message queue, etc.).

In simple terms, WASI-Cloud allows you to do all this:
![untitled](./assets/readme0.png)

Like this:
![untitled](./assets/readme1.png)

## Repository Structure
- `/wit`: the wasi-cloud specification written in `*.wit` format (see [WIT](https://github.com/bytecodealliance/wit-bindgen/blob/main/WIT.md))
- `/src`: the wasi-cloud host cli 
- `/crates`: service implementation
- `/examples`: guest examples
- `/tests`: guest tests

## Looking for Contributors
Do you want to contribute to WASI-Cloud's growth? 

<p align="center">Start with our `CONTRIBUTING.md`</p>

## Build
- Run `make build`

## Run
- Run `make run`
