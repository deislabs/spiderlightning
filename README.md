
<div align="center">
  <h1><code>SpiderLightning</code></h1>
  <img src="docs/images/spiderlightning_logo_alt2.png" width="150px" />
  <p>
    <strong> A set of
    <a href="https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md">WIT</a> interfaces that abstract distributed application capabilities and a runtime CLI for running Wasm applications that use these capabilities. 
    </strong>
  </p>
</div>

> Warning: Alpha quality software, do not use in production.

---

**🚧 Important Development Update 🚧**

As of 01/01/2024, the development of the Slight CLI is on hold. SpiderLightning, as a set of WIT interfaces, is now standardizing as a WASI World under the WASI Subgroup named [wasi-cloud-core](https://github.com/WebAssembly/wasi-cloud-core). Various open-source projects are working to implement host APIs that are part of `wasi-cloud-core`, and we encourage you to check them out:
- [Spin](https://github.com/fermyon/spin)
- [WasmCloud](https://github.com/wasmCloud/wasmCloud)
- [Wasmtime](https://github.com/bytecodealliance/wasmtime), which now ships with a `wasi-http` world.

We appreciate your continued support.

---

## About
SpiderLightning defines a set of [*.wit](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md) files that abstract distributed application capabilities, such as key-value, messaging, http-server/client and more.

Slight CLI is a runtime for running Wasm applications that compile to [WASI](https://wasi.dev/) and use SpiderLightning capabilities.

This repo also contains libraries that implement of SpiderLightning capabilities to common cloud services including AWS, Azure and more.

## Installation

### UNIX

```sh
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/deislabs/spiderlightning/main/install.sh)"
```

### Windows

```sh
iex ((New-Object System.Net.WebClient).DownloadString('https://raw.githubusercontent.com/deislabs/spiderlightning/main/install.ps1'))
```

## Getting Started

`slight` relies on [wit-bindgen v0.2.0](https://github.com/bytecodealliance/wit-bindgen) and currently only supports their language offerings for guest applications (e.g., Rust, C, etc.), plus JavaScript.

### JS

```sh
slight new -n spidey@v0.5.1 js && cd spidey
# ^^^ starts a new js project under SpiderLightning's v0.5.1 spec

slight buildjs -e slightjs_engine.wasm -o main.wasm src/main.js
# ^^^ builds the js application

slight -c slightfile.toml run main.wasm -l
# At this point, you should see: "Hello, JS Wasm!"
```

> Note: All SpiderLightning dependencies are being injected directly into JavaScript's context. This allows you to write SDK-less applications, but, at the same time, it's a can be a bit less clear what functionalities are available to you. For a comprehensive list of examples on how to use SpiderLightning's capabilities in JS, see [here](https://github.com/danbugs/slightjs).

### C

```sh
slight new -n spidey@v0.5.1 c && cd spidey
# ^^^ starts a new c project under SpiderLightning's v0.5.1 spec

# you might want to install wasi-sdk dependencies...
# on unix, run: 
# make install-deps
# on windows, run:
# make install-deps-win

# next, to build...
# on unix, run:
# make bindings && make build
# on windows, run:
# make bindings && make build-win

slight -c slightfile.toml run spidey.wasm
# At this point, you should see: "Hello, SpiderLightning!"
```

### Rust

```sh
slight new -n spidey@v0.5.1 rust && cd spidey
# ^^^ starts a new rust project under SpiderLightning's v0.5.1 spec

cargo build --target wasm32-wasi

slight -c slightfile.toml run target/wasm32-wasi/debug/spidey.wasm
# At this point, you should see: "Hello, SpiderLightning!"
```

## Building C Examples

```sh
git clone https://github.com/deislabs/spiderlightning.git && cd spiderlightning/ # clone our repo locally and go into it
make install-deps # installs the WASI-SDK
make build # builds SpiderLightning/Slight
make build-c # builds our c example
make run-c # runs our c example
```

## Building Rust Examples

There are also Rust examples that can be built (`build-rust`) and ran (`run-rust`). However, we do not recommend running them because some of these examples have dependencies on environment variables or local programs (e.g., `etcd`), so it probably won't work right off the gate like the C one did.

## Repository Structure

- `/crates`: runtime, core library and service implementations
- `/docs`: useful documentation to help you understand design decisions, and how to contribute
- `/examples`: Slight examples
- `/proposals`: design documents
- `/src`: the SpiderLightning cli (i.e., Slight)
- `/templates`: templates used by `slight add` to create a new Rust or C project
- `/tests`: integration tests
- `/wit`: the SpiderLightning specification written in `*.wit` format (see [WIT](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md))

## Looking for Contributors
Do you want to contribute to SpiderLightning's growth? 

<p align="center">Start with our <a href="https://github.com/deislabs/spiderlightning/blob/main/CONTRIBUTING.md">CONTRIBUTING.md</a></p>

Aside from the `CONTRIBUTING.md` doc, here are a few other good starting points to contributing:
- the [`service-implementation-101.md` doc](https://github.com/deislabs/spiderlightning/blob/main/docs/service-implementation-101.md): a step-by-step guide to develop your first service implementor, and
- the [SpiderLightning YouTube Playlist](https://www.youtube.com/playlist?list=PLL6BzOBDywQcXy3otj_Y-20SCSOv-MxT3): a collection of informative and tutorial videos on SpiderLightning/`slight`.

## FAQ

### What problems does SpiderLightning address?

SpiderLightning is a collection of common application interfaces exposed through WebAssembly intended to make cloud-native application development simpler. These interfaces are available to WebAssembly applications through the `slight` Command Line Interface (CLI) and host runtime that implements these interfaces using backing implementations of your choosing. For example, for the "message queue" interface, SpiderLightning may provide backends implemented by Azure Service Bus, Apache Kafka, and more.

SpiderLightning's interfaces offer developers a set of provider-agnostic APIs which enable developers to write portable applications without having to take direct dependencies on vendor specific APIs and SDKs.

Applications targeting SpiderLightning can leverage these interfaces to reduce the amount of code written to achieve tasks such as persisting key/values, participating in pub/sub, handling messages from a message queue, and much more. By reducing the code footprint, SpiderLightning also enables application binaries to be much smaller, often an order of magnitude smaller than similar container-based applications. This feature further increases the portability of applications to target constrained runtime environments like edge devices.

Check out this talk from the [Cloud Native Rejekts](https://youtu.be/zEPeMN0ZlBM?si=0LuOouoLzgpGqqGg) conference for more about SpiderLightning's goals and design.

### What is the difference between SpiderLightning and WebAssembly System Interface (WASI)?

WASI is a set of standardized APIs for Wasm. Its first preview version is a set of POSIX-like APIs to enable Wasm applications to run on a variety of operating systems. WASI Preveiw 2 is much more modular, adds the Wasm Component Model type system and introduces the concept of "worlds" with the [WIT IDL](https://github.com/WebAssembly/component-model/blob/main/design/mvp/WIT.md) to WASI. SpiderLightning could be seen as a WASI World that provides state management, event-driven patterns, and distributed locking.

SpiderLightning has evolved to `wasi-cloud-core` and is now being standardized under the WASI Subgroup. See [here](https://github.com/WebAssembly/wasi-cloud-core)

### What is the difference between SpiderLightning and Dapr?

SpiderLightning and Dapr share the same goal of providing capabilities to distributed applications, but each project's approach to achieving this goal is very different. For example, while Dapr runs as either a sidecar container or one-per-node container and provides an HTTP/gRPC interface for applications, `slight` links applications directly to generated WASM bindings, which means akk calls are executed in-process.

### Why the name "SpiderLightning"?

<img align="right" margin=".1em" src="docs/images/readme2.jpg"/>

Spider Lightning is the name of a phenomenon of "long, horizontally travelling flashes often seen on the underside of [..] clouds" (source: [nssl](https://www.nssl.noaa.gov/education/svrwx101/lightning/types/#:~:text=Spider%20lightning%20refers%20to%20long,often%20linked%20to%20%2BCG%20flashes.)), pictured on the right. From that and from the fact we are developing SpiderLightning/`slight` based off of **Web**Assembly's lightning-fast technologies, the name fit.
