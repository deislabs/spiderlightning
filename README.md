<img align="right" src="docs/images/tmp-logo.png" width="150px" />

# WASI-Cloud
WASI-Cloud defines a set of WebAssembly Interface Types (i.e., WIT) files that abstract the cloud-provider specific knowledge required behind utilizing a Cloud serivce (e.g., key-value store, message queue, etc.).

In simple terms, WASI-Cloud allows you to go:
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
- `/wit`: the wasi-cloud specification written in `*.wit` format (see [WIT](https://github.com/bytecodealliance/wit-bindgen/blob/main/WIT.md))
- `/src`: the wasi-cloud host cli 
- `/crates`: service implementations
- `/examples`: guest examples
- `/tests`: guest tests

## Looking for Contributors
Do you want to contribute to WASI-Cloud's growth? 

<p align="center">Start with our <a href="https://github.com/deislabs/wasi-cloud/blob/main/CONTRIBUTING.md">CONTRIBUTING.md</a></p>

## Build
- Run `make build`

## Run
- Run `make run`
