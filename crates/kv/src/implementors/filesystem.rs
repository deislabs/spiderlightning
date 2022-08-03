use std::{
    fs::{self, File},
    io::{Read, Write},
    path::PathBuf,
    sync::{Arc, Mutex}, env,
};

use anyhow::{Context, Result};
use chrono::Utc;
use crossbeam_channel::Sender;
use events_api::{Event, EventBuilder, EventBuilderV10};
use notify::{Event as NotifyEvent, RecommendedWatcher, RecursiveMode, Watcher};
use uuid::Uuid;

/// This is the underlying struct behind the `Filesystem` variant of the `KvImplementor` enum.
///
/// It provides two properties that pertain solely to the filesystem implementation of
/// of this capability:
///     - `base`, and
///     - `watchers`.
///
/// As per its' usage in `KvImplementor`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
pub struct FilesystemImplementor {
    /// The base path for where the key-value store can be found in your file-system
    pub base: String,
    /// A group of `*Watcher`s that are observing a key
    pub watchers: Vec<Arc<Mutex<RecommendedWatcher>>>,
}

impl FilesystemImplementor {
    pub fn new(name: &str) -> Self {
        Self {
            base: env::temp_dir().join(name).to_str().unwrap().to_owned(),
            watchers: Vec::new(),
        }
    }

    pub fn get(&self, key: &str) -> Result<Vec<u8>> {
        fs::create_dir_all(&self.base)
            .with_context(|| "failed to create base directory for kv store instance")?;
        let mut file =
            File::open(PathBuf::from(&self.base).join(key)).with_context(|| "failed to get key")?;

        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .with_context(|| "failed to read key's value")?;
        Ok(buf)
    }

    pub fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        fs::create_dir_all(&self.base)
            .with_context(|| "failed to create base directory for kv store instance")?;

        let mut file = File::create(PathBuf::from(&self.base).join(key))
            .with_context(|| "failed to create key")?;

        file.write_all(value)
            .with_context(|| "failed to set key's value")?;
        Ok(())
    }

    pub fn delete(&self, key: &str) -> Result<()> {
        fs::create_dir_all(&self.base)
            .with_context(|| "failed to create base directory for kv store instance")?;
        fs::remove_file(PathBuf::from(&self.base).join(key))
            .with_context(|| "failed to delete key's value")?;
        Ok(())
    }

    pub fn watch(&mut self, key: &str, sender: Arc<Mutex<Sender<Event>>>) -> Result<()> {
        let path = PathBuf::from(&self.base).join(key);
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

        // Add a path to be watched — all files and directories at that path and
        // below will be monitored for changes.
        watcher.watch(&path, RecursiveMode::Recursive)?;

        // We don't want to destruct the watcher after the function exits. We
        // want to keep the watcher alive until the resource is dropped.
        self.watchers.push(Arc::new(Mutex::new(watcher)));
        Ok(())
    }
}
