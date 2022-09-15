# `slightfile` Re-Design

## Problem

As of 2022-09-15, the `slightfile` design has two big restrictions:
(1) we do not allow for multiple-capabilities of the same namespace but different implementor types (e.g., `kv.azblob`, and `kv.filesystem`), and
(2) we do not allow for different capabilities to use different secret stores (i.e., we only have one global secret store).

## Solution

This document proposes a re-design of the `slightfile` to:

```toml
specversion = "0.1"
secret_store = "configs.envvars"


[[capability]]
name = "kv.filesystem"
ref = "orders"

[[capability]]
name = "kv.azblob"
ref = "customers"
configs = "configs.azapp"
```

The two changes are:
- we no longer have to use a global `secret_store`, but, rather, we have the option to attach to each capability their own independent config (if none is provided, it can just fallback to the globally defined one),
- we now have a `ref` (or, reference) field to each capability, which, fundamentally, names its' usage.

This reflects no changes in developer code:
```rs
use anyhow::Result;

use kv::*;
wit_bindgen_rust::import!("../../wit/kv.wit");
wit_error_rs::impl_error!(kv::Error);

fn main() -> Result<()> {
    let orders = Kv::open("orders")?; 
    // ^^^ this reflects to an implementor of `kv.filesystem`, with a secret_store of `configs.envvars`
    // - note: the `ref` is equivalent to the folder name in the `kv.filesystem` implementation.
    let customers = Kv::open("customers")?;
    // ^^^ this reflects to an implementor of `kv.azblob`, with configs of `configs.azapp`
    // - note: the `ref` is equivalent to the container name in the `kv.azblob` implementation.
}
```