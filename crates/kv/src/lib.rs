mod providers;

/// The `SCHEME_NAME` defines the name under which a resource is
/// identifiable by in a `ResourceMap`.
const SCHEME_NAME: &str = "kv";

use std::sync::Arc;

use proc_macro_utils::Resource;
use runtime::{impl_resource, resource::ResourceMap};

/// It is mandatory to `use kv::*` due to `impl_resource!`.
/// That is because `impl_resource!` accesses the `crate`'s
/// `add_to_linker` — not `kv::add_to_linker`.
use kv::*;
wit_bindgen_wasmtime::export!("../../wit/kv.wit");
wit_error_rs::impl_error!(kv::Error);
wit_error_rs::impl_from!(anyhow::Error, kv::Error::ErrorWithDescription);

/// The `Kv` structure is what will implement the `kv::Kv` trait
/// coming from the generated code of off `kv.wit`.
///
/// It maintains a `host_state`.
///
/// It implements the `runtime::resource::Resource` trait
/// — an empty trait created for the purpose of granting an
/// explicit way of grouping `struct`s like this one
/// through `dyn Resource`.
#[derive(Resource)]
pub struct Kv {
    host_state: KvState,
}

/// This is the type of the `host_state` property from our `Kv` structure.
///
/// It holds a `resource_map`, which is passed from the `slight` binary
/// and is what helps keep track of what resources have been instantiated.
///
/// It holds a `kv_provider` `String` — this comes directly from a
/// user's `slightfile` and it is what allows us to dynamically
/// dispatch to a specific provider implentaiton.
pub struct KvState {
    resource_map: ResourceMap,
    kv_provider: String,
}

/// This is the type of the associated type coming from the `kv::Kv` trait
/// implementation.
///
/// It holds an `Arc`ed `KvProvider`.
///
/// It must `derive` `Debug` due to a constraint on the associated type.
///
/// It must be public because the implementation of `kv::Kv` cannot leak
/// a private type.
#[derive(Debug)]
pub struct KvInner {
    kv_provider: Arc<KvProvider>,
}

/// This defines the available provider implementations for the `Kv` interface.
///
/// As per its' usage in `KvInner`, it must `derive` `Debug`.
#[derive(Debug)]
enum KvProvider {
    Filesystem,
    AzBlob,
}

// This implements the `ResourceBuilder` trait for our `Kv` `struct`,
// and `ResourceTables` for our `kv::KvTables` object.
//
// The `ResourceBuilder` trait provides two functions:
// - `add_to_linker`, and
// - `builda_data`.
impl_resource!(Kv, kv::KvTables<Kv>, KvState, SCHEME_NAME.to_string());

/// This is the implementation for the generated `kv::Kv` trait from the `kv.wit` file.
impl kv::Kv for Kv {
    type Kv = KvInner;

    fn kv_open(&mut self, name: &str) -> Result<Self::Kv, Error> {
        todo!()
    }

    fn kv_get(&mut self, self_: &Self::Kv, key: &str) -> Result<PayloadResult, Error> {
        todo!()
    }

    fn kv_set(
        &mut self,
        self_: &Self::Kv,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        todo!()
    }

    fn kv_delete(&mut self, self_: &Self::Kv, key: &str) -> Result<(), Error> {
        todo!()
    }

    fn kv_watch(&mut self, self_: &Self::Kv, key: &str) -> Result<Observable, Error> {
        todo!()
    }
}
