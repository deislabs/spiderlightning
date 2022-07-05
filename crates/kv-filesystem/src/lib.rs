use anyhow::{Context, Result};
use notify::{Event as NotifyEvent, FsEventWatcher, RecursiveMode, Watcher};
use proc_macro_utils::RuntimeResource;
use runtime::resource::Event;
use runtime::resource::{get, Ctx, DataT, Linker, Map, Resource, ResourceMap, RuntimeResource};
use std::sync::{Arc, Mutex};

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
#[derive(Default, Clone, RuntimeResource)]
pub struct KvFilesystem {
    /// The root directory of the filesystem
    inner: Option<String>,
    resource_map: Option<ResourceMap>,
    wathchers: Vec<Arc<Mutex<FsEventWatcher>>>,
}

impl kv::Kv for KvFilesystem {
    /// Contruct a new `KvFilesystem` from a folder name. This folder will be created under `/tmp`
    fn get_kv(&mut self, name: &str) -> Result<ResourceDescriptorResult, Error> {
        let path = Path::new("/tmp").join(name);
        let path = path
            .to_str()
            .with_context(|| format!("failed due to invalid path: {}", name))?
            .to_string();
        self.inner = Some(path);

        let rd = Uuid::new_v4().to_string();
        let cloned = self.clone(); // have to clone here because of the mutable borrow below
        let mut map = Map::lock(&mut self.resource_map)?;
        map.set(rd.clone(), (Box::new(cloned), None));
        Ok(rd)
    }

    /// Output the value of a set key
    fn get(&mut self, rd: ResourceDescriptorParam, key: &str) -> Result<PayloadResult, Error> {
        Uuid::parse_str(rd).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::lock(&mut self.resource_map)?;
        let base = map.get::<String>(rd)?;
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
    fn set(
        &mut self,
        rd: ResourceDescriptorParam,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        Uuid::parse_str(rd).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::lock(&mut self.resource_map)?;
        let base = map.get::<String>(rd)?;

        fs::create_dir_all(&base)
            .with_context(|| "failed to create base directory for kv store instance")?;

        let mut file =
            File::create(PathBuf::from(base).join(key)).with_context(|| "failed to create key")?;

        file.write_all(value)
            .with_context(|| "failed to set key's value")?;
        Ok(())
    }

    /// Delete a key-value pair
    fn delete(&mut self, rd: ResourceDescriptorParam, key: &str) -> Result<(), Error> {
        Uuid::parse_str(rd).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::lock(&mut self.resource_map)?;
        let base = map.get::<String>(rd)?;

        fs::create_dir_all(&base)
            .with_context(|| "failed to create base directory for kv store instance")?;
        fs::remove_file(PathBuf::from(base).join(key))
            .with_context(|| "failed to delete key's value")?;
        Ok(())
    }

    fn watch(&mut self, rd: ResourceDescriptorParam, key: &str) -> Result<Observable, Error> {
        Ok(Observable {
            rd: rd.to_string(),
            key: key.to_string(),
        })
    }
}

impl Resource for KvFilesystem {
    fn add_resource_map(&mut self, resource_map: ResourceMap) -> Result<()> {
        self.resource_map = Some(resource_map);
        Ok(())
    }

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
                    let event = Event::new(
                        path,
                        format!("{:#?}", event.kind),
                        "1.0".to_string(),
                        id,
                        Some(key.clone()),
                    );
                    sender
                        .lock()
                        .unwrap()
                        .send(event)
                        .expect("internal error: send");
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
