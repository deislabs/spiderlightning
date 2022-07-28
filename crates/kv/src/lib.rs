mod providers;

/// The `SCHEME_NAME` defines the name under which a resource is
/// identifiable by in a `ResourceMap`.
const SCHEME_NAME: &str = "kv";

use std::{sync::{Arc, Mutex}, path::PathBuf};

use chrono::Utc;
use crossbeam_channel::Sender;
use events_api::{Event, EventBuilderV10, EventBuilder};
use notify::{Event as NotifyEvent, RecommendedWatcher, RecursiveMode, Watcher};
use uuid::Uuid;
use anyhow::{Result, Context};

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
/// It holds a `kv_provider` (i.e., a variant `KvProvider` `enum`).
///
/// It must `derive`:
///     - `Debug` due to a constraint on the associated type.
///     - `Clone` because the `ResourceMap` it will be added onto,
///     must own its' data. 
///
/// It must be public because the implementation of `kv::Kv` cannot leak
/// a private type.
#[derive(Debug, Clone)]
pub struct KvInner {
    kv_provider: KvProvider,
}

impl runtime::resource::Watch for KvInner {
    fn watch(&mut self, key: &str, sender: Arc<Mutex<Sender<Event>>>) -> Result<()> {
        if let KvProvider::Filesystem { base, watchers } = &mut self.kv_provider {
            let path = PathBuf::from(base.as_ref().unwrap()).join(key);
            let key = key.to_string();
            let mut watcher =
                notify::recommended_watcher(move |res: Result<NotifyEvent, _>| match res {
                    Ok(event) => {
                        // we use uuid to identify an event
                        let id = Uuid::new_v4().to_string();
                        let path = event
                            .paths
                            .get(0)
                            .map(|x| format!("{}", x.display()))
                            .unwrap_or_default();
                        let content_type = "application/json";
                        let data = serde_json::json!({ "key": key });
                        let event = EventBuilderV10::new()
                            .id(id)
                            .source(path)
                            .ty(format!("{:#?}", event.kind))
                            .time(Utc::now())
                            .data(content_type, data)
                            .build()
                            .with_context(|| "failed to build event")
                            .unwrap_or_else(|e| {
                                tracing::error!("failed to build event: {}, sending default event", e);
                                Event::default()
                            });
                        sender
                            .lock()
                            .unwrap()
                            .send(event)
                            .with_context(|| "internal error: send")
                            .unwrap_or_else(|e| {
                                tracing::error!("failed to send event: {}", e);
                                panic!("internal error: failed to send event")
                            });
                    }
                    Err(e) => println!("watch error: {:?}", e),
                })?;
        
            // Add a path to be watched. All files and directories at that path and
            // below will be monitored for changes.
            watcher.watch(&path, RecursiveMode::Recursive)?;
            // we don't want to destruct the watcher after the function exit. We
            // want to keep the watcher alive until the resource is dropped.
            watchers.as_mut().unwrap().push(Arc::new(Mutex::new(watcher)));
            Ok(())
        } else {
            todo!()
        }
    }
}

/// This defines the available provider implementations for the `Kv` interface.
///
/// As per its' usage in `KvInner`, it must `derive` `Debug`.
#[derive(Debug, Clone)]
enum KvProvider {
    Filesystem {
        base: Option<String>,
        watchers: Option<Vec<Arc<Mutex<RecommendedWatcher>>>>,
    },
    AzBlob,
}

impl From<&str> for KvProvider {
    fn from(from: &str) -> Self {
        match from {
            "kv.filesystem" => Self::Filesystem { base: None, watchers: None},
            "kv.azblob" => Self::AzBlob,
            _ => panic!("failed to match provided kv name to any known host implementations")
        }
    }
}

impl From<KvProvider> for String {
    fn from(from: KvProvider) -> Self {
        match from {
            KvProvider::Filesystem { base, watchers } => "kv.filesystem".to_string(),
            KvProvider::AzBlob => "kv.azblob".to_string()
        }
    }
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

    fn kv_open(&mut self, _name: &str) -> Result<Self::Kv, Error> {
        // populate our inner kv object w/ the state received from `slight`
        // — in this case, that is: what type of kv provider we are using.
        let inner = Self::Kv {
            kv_provider: self.host_state.kv_provider.as_str().into()
        };
        let rd = Uuid::new_v4().to_string();
        self.host_state.resource_map.lock().unwrap().set(rd, Box::new(inner.clone()));
        todo!()
    }

    fn kv_get(&mut self, _self_: &Self::Kv, _key: &str) -> Result<PayloadResult, Error> {
        todo!()
    }

    fn kv_set(
        &mut self,
        _self_: &Self::Kv,
        _key: &str,
        _value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        todo!()
    }

    fn kv_delete(&mut self, _self_: &Self::Kv, _key: &str) -> Result<(), Error> {
        todo!()
    }

    fn kv_watch(&mut self, _self_: &Self::Kv, _key: &str) -> Result<Observable, Error> {
        todo!()
    }
}
