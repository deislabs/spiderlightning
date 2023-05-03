
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
slight new -n spidey@v0.5.0 js
# ^^^ starts a new js project under SpiderLightning's v0.5.0 spec

slight buildjs -e slightjs_engine.wasm -o main.wasm src/main.js
# ^^^ builds the js application

slight -c slightfile.toml run -m main.wasm -l
# At this point, you should see: "Hello, JS Wasm!"
```

> Note: All SpiderLightning dependencies are being injected directly into JavaScript's context. This allows you to write SDK-less applications, but, at the same time, it's a can be a bit less clear what functionalities are available to you. For a comprehensive list of examples on how to use SpiderLightning's capabilities in JS, see [here](https://github.com/danbugs/slightjs).

### C

```sh
slight new -n spidey@v0.5.0 c
# ^^^ starts a new c project under SpiderLightning's v0.5.0 spec

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

slight -c slightfile.toml run -m spidey.wasm
# At this point, you should see: "Hello, SpiderLightning!"
```

### Rust

```sh
slight new -n spidey@v0.5.0 rust && cd spidey
# ^^^ starts a new rust project under SpiderLightning's v0.5.0 spec

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

### What is the difference between SpiderLightning and WebAssembly System Interface (WASI)?

WASI's primary goal is to provide operating system abstractions for Wasm applications running outside of browser environments. SpiderLightning interfaces extend WASI to provide application capabilities, such as state management, event-driven patterns, and distributed locking. In the future, we hope that SpiderLightning's interfaces or interfaces that provide similar capabilities become part of WASI specification.

### What is the difference between SpiderLightning and Dapr?

SpiderLightning and Dapr share the same goal of providing capabilities to distributed applications, but each project's approach to achieving this goal is very different. For example, while Dapr runs as a sidecar container and provides an HTTP/gRPC interface for applications, SpiderLightning chooses not take a dependency on network transport protocols. Instead, `slight` links applications directly to generated WASM bindings, then executes them directly.

### Why the name "SpiderLightning"?

<img align="right" margin=".1em" src="docs/images/readme2.jpg"/>

Spider Lightning is the name of a phenomenon of "long, horizontally travelling flashes often seen on the underside of [..] clouds" (source: [nssl](https://www.nssl.noaa.gov/education/svrwx101/lightning/types/#:~:text=Spider%20lightning%20refers%20to%20long,often%20linked%20to%20%2BCG%20flashes.)), pictured on the right. From that and from the fact we are developing SpiderLightning/`slight` based off of **Web**Assembly's lightning-fast technologies, the name fit.
