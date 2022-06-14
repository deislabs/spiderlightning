use anyhow::Result;
use mq::*;
use runtime::resource::{get, Context, DataT, HostResource, Linker, Resource, ResourceMap};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

wit_bindgen_wasmtime::export!("../../wit/mq.wit");

const SCHEME_NAME: &str = "mq";

/// A Filesystem implementation for mq interface.
#[derive(Clone)]
pub struct MqFilesystem {
    queue: String,
    path: String,
    resource_map: Option<ResourceMap>,
}

impl Default for MqFilesystem {
    fn default() -> Self {
        Self {
            queue: ".queue".to_string(),
            path: String::default(),
            resource_map: None,
        }
    }
}

impl mq::Mq for MqFilesystem {
    fn get_mq(&mut self, name: &str) -> Result<ResourceDescriptorResult, Error> {
        let path = Path::new("/tmp").join(name);
        let path = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("invalid path: {}", name))?
            .to_string();
        self.path = path;

        let uuid = Uuid::new_v4();
        let rd = uuid.to_string();

        let cloned = self.clone();
        let mut map = self
            .resource_map
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        map.set(rd.clone(), Box::new(cloned))?;

        Ok(rd)
    }

    fn send(&mut self, rd: ResourceDescriptorParam, msg: PayloadParam<'_>) -> Result<(), Error> {
        if Uuid::parse_str(rd).is_err() {
            return Err(Error::DescriptorError);
        }

        let map = self
            .resource_map
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        let base = map.get::<String>(rd)?;

        // get a random name for a queue element
        let rand_file_name = format!(
            "{:?}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
        );

        fs::create_dir_all(&base)?;

        // create a file to store the queue element data
        let mut file = File::create(path(&rand_file_name, base))?;

        // write to file msg sent
        file.write_all(msg)?;

        // open/create queue and store one random name for a queue element per line
        let mut queue = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path(&self.queue, base))?;

        // add queue element name to the bottom of the queue
        writeln!(queue, "{}", rand_file_name)?;

        Ok(())
    }

    fn receive(&mut self, rd: ResourceDescriptorParam) -> Result<PayloadResult, Error> {
        if Uuid::parse_str(rd).is_err() {
            return Err(Error::DescriptorError);
        }

        let map = self
            .resource_map
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("resource map is not initialized"))?
            .lock()
            .unwrap();
        let base = map.get::<String>(rd)?;

        fs::create_dir_all(&base)?;

        // get the queue
        let queue = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path(&self.queue, base))?;

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
            fs::write(path(&self.queue, base), queue_post_receive)?;

            // remove \n char from end of queue element
            to_receive.pop();

            // get element at top of queue
            let mut file = File::open(path(&to_receive, base))?;
            let mut buf = Vec::new();

            // read element's message
            file.read_to_end(&mut buf)?;

            // clean-up element from disk
            fs::remove_file(path(&to_receive, base))?;

            Ok(buf)
        } else {
            // if queue is empty, respond with empty string
            Ok(Vec::new())
        }
    }
}

impl Resource for MqFilesystem {
    fn add_resource_map(&mut self, resource_map: ResourceMap) -> Result<()> {
        self.resource_map = Some(resource_map);
        Ok(())
    }

    fn get_inner(&self) -> &dyn std::any::Any {
        &self.path // TODO: do we need to put queue name here?
    }
}

impl HostResource for MqFilesystem {
    fn add_to_linker(linker: &mut Linker<Context<DataT>>) -> Result<()> {
        crate::add_to_linker(linker, |cx| get::<Self>(cx, SCHEME_NAME.to_string()))
    }

    fn build_data() -> Result<DataT> {
        let mq_filesystem = Self::default();
        Ok(Box::new(mq_filesystem))
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
