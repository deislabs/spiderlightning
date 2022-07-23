use anyhow::{Context, Result};
use events_api::{Event, EventBuilder, EventBuilderV10};
use notify::{Event as NotifyEvent, RecommendedWatcher, RecursiveMode, Watcher};
use proc_macro_utils::Resource;
use runtime::impl_resource;
use runtime::resource::{
    get_table, Ctx, HostState, Linker, Resource, ResourceBuilder, ResourceMap, ResourceTables,
    Watch,
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

use state_store::*;

wit_bindgen_wasmtime::export!("../../wit/state_store.wit");
wit_error_rs::impl_error!(state_store::Error);
wit_error_rs::impl_from!(anyhow::Error, state_store::Error::ErrorWithDescription);

const SCHEME_NAME: &str = "state_store.filesystem";

/// A Filesystem implementation for the state_store interface
#[derive(Default, Clone, Resource)]
pub struct StateStoreFilesystem {
    /// The host state. Currently this is a map of resource names to resource descriptors.
    /// If there are more host-specified states, they can be added here.
    host_state: ResourceMap,
}

impl_resource!(
    StateStoreFilesystem,
    state_store::StateStoreTables<StateStoreFilesystem>,
    ResourceMap,
    SCHEME_NAME.to_string()
);

/// A Filesystem implementation for the state_store interface
#[derive(Default, Debug, Clone)]
pub struct StateStoreFilesystemInner {
    /// The root directory of the filesystem
    base: String,

    /// A list of watchers for the filesystem
    watchers: Vec<Arc<Mutex<RecommendedWatcher>>>,

    /// resource descriptor
    rd: String,
}

impl StateStoreFilesystemInner {
    /// Create a new StateStoreFilesystemInner
    pub fn new(base: String) -> Self {
        Self {
            base,
            watchers: Vec::new(),
            rd: Uuid::new_v4().to_string(),
        }
    }
}

impl state_store::StateStore for StateStoreFilesystem {
    type StateStore = StateStoreFilesystemInner;
    /// Contruct a new `StateStoreFilesystem` from a folder name. This folder will be created under `/tmp`
    fn state_store_open(&mut self, name: &str) -> Result<Self::StateStore, Error> {
        let path = Path::new("/tmp").join(name);
        let path = path
            .to_str()
            .with_context(|| format!("failed due to invalid path: {}", name))?
            .to_string();

        let state_store_guest = Self::StateStore::new(path);
        self.host_state.lock().unwrap().set(
            state_store_guest.rd.clone(),
            Box::new(state_store_guest.clone()),
        );
        Ok(state_store_guest)
    }

    /// Output the value of a set key
    fn state_store_get(
        &mut self,
        self_: &Self::StateStore,
        key: &str,
    ) -> Result<PayloadResult, Error> {
        let base = &self_.base;
        fs::create_dir_all(&base)
            .with_context(|| "failed to create base directory for state store instance")?;
        let mut file =
            File::open(PathBuf::from(base).join(key)).with_context(|| "failed to get key")?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .with_context(|| "failed to read key's value")?;
        Ok(buf)
    }

    /// Create a key-value pair
    fn state_store_set(
        &mut self,
        self_: &Self::StateStore,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        let base = &self_.base;
        fs::create_dir_all(base)
            .with_context(|| "failed to create base directory for state store instance")?;

        let mut file =
            File::create(PathBuf::from(base).join(key)).with_context(|| "failed to create key")?;

        file.write_all(value)
            .with_context(|| "failed to set key's value")?;
        Ok(())
    }

    /// Delete a key-value pair
    fn state_store_delete(&mut self, self_: &Self::StateStore, key: &str) -> Result<(), Error> {
        let base = &self_.base;

        fs::create_dir_all(base)
            .with_context(|| "failed to create base directory for state store instance")?;
        fs::remove_file(PathBuf::from(base).join(key))
            .with_context(|| "failed to delete key's value")?;
        Ok(())
    }

    /// Watch for changes to a key-value pair
    fn state_store_watch(
        &mut self,
        self_: &Self::StateStore,
        key: &str,
    ) -> Result<Observable, Error> {
        Ok(Observable::new(&self_.rd, key))
    }
}

impl Watch for StateStoreFilesystemInner {
    fn watch(&mut self, key: &str, sender: Arc<Mutex<Sender<Event>>>) -> Result<()> {
        let path = path(key, &self.base);
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
        self.watchers.push(Arc::new(Mutex::new(watcher)));
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
