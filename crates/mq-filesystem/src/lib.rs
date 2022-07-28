use anyhow::{Context, Result};
use crossbeam_channel::Sender;
use events_api::Event;
use mq::*;
use proc_macro_utils::{Resource, Watch};
use runtime::impl_resource;
use runtime::resource::{
    ResourceMap,
    Watch,
};
use std::sync::{Arc, Mutex};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

wit_bindgen_wasmtime::export!("../../wit/mq.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);
wit_error_rs::impl_from!(std::io::Error, Error::ErrorWithDescription);

const SCHEME_NAME: &str = "mq.filesystem";

/// A Filesystem implementation for the mq interface.
#[derive(Clone, Resource, Default)]
pub struct MqFilesystem {
    host_state: ResourceMap,
}

impl_resource!(
    MqFilesystem,
    mq::MqTables<MqFilesystem>,
    ResourceMap,
    SCHEME_NAME.to_string()
);

#[derive(Clone, Debug, Watch)]
pub struct MqFileSystemInner {
    queue: String,
    base: String,
}

// vvv we implement default manually because of the `queue` attribute
impl MqFileSystemInner {
    fn new(base: String) -> Self {
        Self {
            queue: ".queue".to_string(),
            base,
        }
    }
}

impl mq::Mq for MqFilesystem {
    type Mq = MqFileSystemInner;
    /// Construct a new `MqFilesystem` instance provided a folder name. The folder will be created under `/tmp`.
    fn mq_open(&mut self, name: &str) -> Result<Self::Mq, Error> {
        let path = Path::new("/tmp").join(name);
        let path = path
            .to_str()
            .with_context(|| format!("failed due to invalid path: {}", name))?
            .to_string();

        let mq_fs_guest = Self::Mq::new(path);

        let rd = Uuid::new_v4().to_string();
        self.host_state
            .lock()
            .unwrap()
            .set(rd, Box::new(mq_fs_guest.clone()));
        Ok(mq_fs_guest)
    }

    /// Send a message to the message queue
    fn mq_send(&mut self, self_: &Self::Mq, msg: PayloadParam<'_>) -> Result<(), Error> {
        let base = &self_.base;
        let queue = &self_.queue;

        // get a random name for a queue element
        let rand_file_name = format!(
            "{:?}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
        );

        fs::create_dir_all(base)?;

        // create a file to store the queue element data
        let mut file = File::create(PathBuf::from(base).join(&rand_file_name))?;

        // write to file msg sent
        file.write_all(msg)?;

        // open/create queue and store one random name for a queue element per line
        let mut queue = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(PathBuf::from(base).join(queue))?;

        // add queue element name to the bottom of the queue
        writeln!(queue, "{}", rand_file_name)?;

        Ok(())
    }

    /// Receive a message from the message queue
    fn mq_receive(&mut self, self_: &Self::Mq) -> Result<PayloadResult, Error> {
        let base = &self_.base;

        fs::create_dir_all(base)?;

        // get the queue
        let queue = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(PathBuf::from(base).join(&self_.queue))?;

        if queue.metadata().unwrap().len() != 0 {
            // get top element in the queue
            let mut queue_buffer = BufReader::new(&queue);
            let mut to_receive: String = String::from("");
            queue_buffer.read_line(&mut to_receive)?;

            // get queue after receive
            let mut queue_post_receive = queue_buffer
                .lines()
                .map(|x| x.unwrap())
                .collect::<Vec<String>>()
                .join("\n");

            // if queue isn't empty, we want to append a new-line char for subsequent send
            if !queue_post_receive.is_empty() {
                queue_post_receive += "\n";
            }

            // update queue status
            fs::write(PathBuf::from(base).join(&self_.queue), queue_post_receive)?;

            // remove \n char from end of queue element
            to_receive.pop();

            // get element at top of queue
            let mut file = File::open(PathBuf::from(base).join(&to_receive))?;
            let mut buf = Vec::new();

            // read element's message
            file.read_to_end(&mut buf)?;

            // clean-up element from disk
            fs::remove_file(PathBuf::from(base).join(&to_receive))?;

            Ok(buf)
        } else {
            // if queue is empty, respond with empty string
            Ok(Vec::new())
        }
    }
}
