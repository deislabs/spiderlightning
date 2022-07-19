use anyhow::{Context, Result};
use events_api::{Event, EventBuilder, EventBuilderV10};
use notify::{Event as NotifyEvent, RecommendedWatcher, RecursiveMode, Watcher};
use runtime::impl_resource;
use runtime::resource::{
    get_table, Ctx, DataT, Linker, Map, Resource, ResourceMap, ResourceTables, RuntimeResource,
};
use std::sync::{Arc, Mutex};

use chrono::Utc;
use crossbeam_channel::Sender;
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;

use kv::*;

wit_bindgen_wasmtime::export!("../../wit/kv.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);

const SCHEME_NAME: &str = "filekv";

/// A Filesystem implementation for the kv interface
#[derive(Default, Clone)]
pub struct KvFilesystem {
    /// The root directory of the filesystem
    inner: Option<String>,
    /// The host state. Currently this is a map of resource names to resource descriptors.
    /// If there are more host-specified states, they can be added here.
    host_state: Option<ResourceMap>,
    wathchers: Vec<Arc<Mutex<RecommendedWatcher>>>,
}

impl_resource!(
    KvFilesystem,
    kv::KvTables<KvFilesystem>,
    ResourceMap,
    SCHEME_NAME.to_string()
);

impl kv::Kv for KvFilesystem {
    type Kv = String;
    /// Contruct a new `KvFilesystem` from a folder name. This folder will be created under `/tmp`
    fn kv_open(&mut self, name: &str) -> Result<Self::Kv, Error> {
        let path = Path::new("/tmp").join(name);
        let path = path
            .to_str()
            .with_context(|| format!("failed due to invalid path: {}", name))?
            .to_string();
        self.inner = Some(path);

        let rd = Uuid::new_v4().to_string();
        let cloned = self.clone(); // have to clone here because of the mutable borrow below
        let mut map = Map::lock(&mut self.host_state)?;
        map.set(rd.clone(), (Box::new(cloned), None));
        Ok(rd)
    }

    /// Output the value of a set key
    fn kv_get(&mut self, self_: &Self::Kv, key: &str) -> Result<PayloadResult, Error> {
        Uuid::parse_str(self_)
            .with_context(|| "internal error: failed to parse internal handle to this resource")?;

        let map = Map::lock(&mut self.host_state)?;
        let base = map.get::<String>(self_)?;
        fs::create_dir_all(&base)
            .with_context(|| "failed to create base directory for kv store instance")?;
        let mut file =
            File::open(PathBuf::from(base).join(key)).with_context(|| "failed to get key")?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .with_context(|| "failed to read key's value")?;
        Ok(buf)
    }

    /// Create a key-value pair
    fn kv_set(
        &mut self,
        self_: &Self::Kv,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        Uuid::parse_str(self_)
            .with_context(|| "internal error: failed to parse internal handle to this resource")?;

        let map = Map::lock(&mut self.host_state)?;
        let base = map.get::<String>(self_)?;

        fs::create_dir_all(&base)
            .with_context(|| "failed to create base directory for kv store instance")?;

        let mut file =
            File::create(PathBuf::from(base).join(key)).with_context(|| "failed to create key")?;

        file.write_all(value)
            .with_context(|| "failed to set key's value")?;
        Ok(())
    }

    /// Delete a key-value pair
    fn kv_delete(&mut self, self_: &Self::Kv, key: &str) -> Result<(), Error> {
        Uuid::parse_str(self_)
            .with_context(|| "internal error: failed to parse internal handle to this resource")?;

        let map = Map::lock(&mut self.host_state)?;
        let base = map.get::<String>(self_)?;

        fs::create_dir_all(&base)
            .with_context(|| "failed to create base directory for kv store instance")?;
        fs::remove_file(PathBuf::from(base).join(key))
            .with_context(|| "failed to delete key's value")?;
        Ok(())
    }

    fn kv_watch(&mut self, self_: &Self::Kv, key: &str) -> Result<Observable, Error> {
        Ok(Observable::new(self_, key))
    }
}

impl Resource for KvFilesystem {
    fn get_inner(&self) -> &dyn std::any::Any {
        self.inner.as_ref().unwrap()
    }

    fn watch(
        &mut self,
        base: &str,
        _rd: &str,
        key: &str,
        sender: Arc<Mutex<Sender<Event>>>,
    ) -> Result<()> {
        let path = path(key, base);
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
        self.wathchers.push(Arc::new(Mutex::new(watcher)));
        Ok(())
    }
}

/// Return the absolute path for the file corresponding to the given key.
fn path(name: &str, base: &str) -> PathBuf {
    PathBuf::from(base).join(name)
}

impl Observable {
    pub fn new(rd: &str, key: &str) -> Self {
        Observable {
            rd: rd.to_string(),
            key: key.to_string(),
        }
    }
}
