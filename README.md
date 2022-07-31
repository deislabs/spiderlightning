<img align="right" src="docs/images/tmp-logo.png" width="150px" />

# SpiderLightning (or, `slight`)
SpiderLightning defines a set of WebAssembly Interface Types (i.e., WIT) files that abstract distributed application capabilities, such as state management, pub/sub, event driven programming, and more.  

In simple terms, SpiderLightning allows you to go:
<table>
<tr>
    <th>From this:</th>
    <th>To this:</th>
</tr>
<tr>
    <td><img src="docs/images/readme0.png"/></td>
    <td><img src="docs/images/readme1.png"/></td>
</tr>
</table>


## Repository Structure
- `/wit`: the SpiderLightning specification written in `*.wit` format (see [WIT](https://github.com/bytecodealliance/wit-bindgen/blob/main/WIT.md))
- `/src`: the SpiderLightning host cli (i.e., Slight)
- `/crates`: service implementations
- `/examples`: guest examples
- `/tests`: guest tests

## Looking for Contributors
Do you want to contribute to SpiderLightning's growth? 

<p align="center">Start with our <a href="https://github.com/deislabs/spiderlightning/blob/main/CONTRIBUTING.md">CONTRIBUTING.md</a></p>

## Getting Started

```sh
$ git clone https://github.com/deislabs/spiderlightning.git && cd spiderlightning/ # clone our repo locally and go into it
$ make install-deps # installs the WASI-SDK
$ make build # builds SpiderLightning/Slight
$ make build-c # builds our c example
$ make run-c # runs our c example
```

> Note: There are also Rust examples that can be built (`build-rust`) and ran (`run-rust`). However, some of these examples have some dependencies on environment variables or local programs (e.g., `etcd`), so it probably won't work right off the gate like the C one did.


## FAQ

**What problems does SpiderLightning address?**

Spiderlightning is a collection of common application interfaces exposed through WebAssembly that are intended to make cloud-native application development simpler. Spiderlightning interfaces are available to WebAssembly applications through the `slight` Command Line Interface (CLI) and host runtime.

Spiderlightning's interfaces offer developers a provider agnostic set of APIs which enable developers to write portable applications without having to take dependencies on vendor specific APIs and SDKs.

Applications targeting Spiderlightning can leverage these interfaces to reduce the amount of code written to achieve tasks such as persisting key/values, participating in pub/sub, handling messages from message queue, and much more. By reducing the code footprint and corresponding Wasm binary, Spiderlightning also enables applications to be much lighter weight, often an order of magnitude lighter weight than corresponding container based applications. This further increases the portability of applications to target constrained runtime environments like edge and small devices.

**What is the difference between SpiderLightning and WebAssembly System Interface (WASI)?**

WASI's primary goal is to provide operating system abstractions for Wasm applications running outside of browser environments. SpiderLightning interfaces extend WASI's goal to provide application capabilities, such as state management, event-driven pattern, or distributed locking. In the future, we hope that SpiderLightning interfaces become part of WASI specification. 

**What is the difference between SpiderLightning and Dapr?**

SpiderLightning and Dapr share the same goal of providing capabilities to distributed applications, but each products' approach to a solution is very different. That is, while Dapr runs as a sidecar using HTTP/gRPC to communicate with applications, SpiderLightning's chooses not take a dependency on network transport protocols. Instead, `slight`, for example, uses generated Wasm bindings to directly execute Wasm applications.
