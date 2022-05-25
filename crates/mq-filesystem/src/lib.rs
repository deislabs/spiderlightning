use anyhow::{bail, Result};
use mq::*;
use runtime::resource::{get, Context, DataT, HostResource, Linker, Resource, ResourceTables};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use url::Url;

wit_bindgen_wasmtime::export!("../../wit/mq.wit");

/// A Filesystem implementation for mq interface.
#[derive(Default)]
pub struct MqFilesystem {
    queue: String,
    path: String,
}

impl MqFilesystem {
    /// Create a new MqFilesystem.
    pub fn new(path: String) -> Self {
        Self {
            queue: ".queue".to_string(),
            path,
        }
    }
}

impl mq::Mq for MqFilesystem {
    type ResourceDescriptor = u64;

    fn get_mq(&mut self) -> Result<Self::ResourceDescriptor, Error> {
        Ok(0)
    }

    fn send(&mut self, rd: &Self::ResourceDescriptor, msg: PayloadParam<'_>) -> Result<(), Error> {
        if *rd != 0 {
            return Err(Error::OtherError);
        }

        // get a random name for a queue element
        let rand_file_name = format!(
            "{:?}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
        );

        // create a file to store the queue element data
        let mut file = File::create(path(&rand_file_name, &self.path))?;

        // write to file msg sent
        file.write_all(msg)?;

        // open/create queue and store one random name for a queue element per line
        let mut queue = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path(&self.queue, &self.path))?;

        // add queue element name to the bottom of the queue
        writeln!(queue, "{}", rand_file_name)?;

        Ok(())
    }

    fn receive(&mut self, rd: &Self::ResourceDescriptor) -> Result<PayloadResult, Error> {
        if *rd != 0 {
            return Err(Error::OtherError);
        }

        // get the queue
        let queue = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path(&self.queue, &self.path))?;

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
            fs::write(path(&self.queue, &self.path), queue_post_receive)?;

            // remove \n char from end of queue element
            to_receive.pop();

            // get element at top of queue
            let mut file = File::open(path(&to_receive, &self.path))?;
            let mut buf = Vec::new();

            // read element's message
            file.read_to_end(&mut buf)?;

            // clean-up element from disk
            fs::remove_file(path(&to_receive, &self.path))?;

            Ok(buf)
        } else {
            // if queue is empty, respond with empty string
            Ok(Vec::new())
        }
    }
}

impl<T> ResourceTables<dyn Resource> for MqTables<T> where T: mq::Mq + 'static {}

impl Resource for MqFilesystem {
    fn from_url(url: Url) -> Result<Self> {
        let path = url.to_file_path();
        match path {
            Ok(path) => {
                let path = path.to_str().unwrap_or(".").to_string();
                Ok(Self::new(path))
            }
            Err(_) => bail!("invalid url: {}", url),
        }
    }
}

impl HostResource for MqFilesystem {
    fn add_to_linker(linker: &mut Linker<Context<DataT>>) -> Result<()> {
        crate::add_to_linker(linker, get::<Self, MqTables<Self>>)
    }

    fn build_state(url: Url) -> Result<DataT> {
        let mq_filesystem = Self::from_url(url)?;
        Ok((
            Box::new(mq_filesystem),
            Box::new(MqTables::<Self>::default()),
        ))
    }
}

/// TODO (Dan): This function is used across kv-filesystem and mq-filesystem — we might want to make a utils crate.
/// Return the absolute path for the file corresponding to the given key.
fn path(name: &str, base: &str) -> PathBuf {
    PathBuf::from(base).join(name)
}

impl From<anyhow::Error> for Error {
    fn from(_: anyhow::Error) -> Self {
        Self::OtherError
    }
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::OtherError
    }
}
