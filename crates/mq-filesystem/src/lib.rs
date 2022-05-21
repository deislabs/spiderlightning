use anyhow::{bail, Result};
use capability::{Resource, ResourceTables};
use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use url::Url;

pub use mq::add_to_linker;
use mq::*;

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
            return Err(Error::GenericError);
        }

        let rand_file_name = gen_rand_name();

        let mut file = File::create(path(&rand_file_name, &self.path))?;
        file.write_all(msg)?;

        let mut queue = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path(&self.queue, &self.path))
            .unwrap();

        writeln!(queue, "{}", rand_file_name)?;

        Ok(())
    }

    fn receive(&mut self, rd: &Self::ResourceDescriptor) -> Result<PayloadResult, Error> {
        if *rd != 0 {
            return Err(Error::GenericError);
        }

        let queue_pre_receive = OpenOptions::new()
            .read(true)
            .create(true)
            .write(true)
            .open(path(&self.queue, &self.path))
            .expect("error opening queue");

        if queue_pre_receive.metadata().unwrap().len() != 0 {
            let mut queue_buffer = BufReader::new(queue_pre_receive);
            let mut to_receive = "".to_string();
            let _ = queue_buffer.read_line(&mut to_receive);

            let queue_post_receive = OpenOptions::new()
                .read(true)
                .write(true)
                .open(path(&self.queue, &self.path))
                .expect("error opening queue");

            let mut queue_re_write = BufReader::new(queue_post_receive)
                .lines()
                .skip(1)
                .map(|x| x.unwrap())
                .collect::<Vec<String>>()
                .join("\n");

            queue_re_write = if !queue_re_write.is_empty() {
                queue_re_write + "\n"
            } else {
                queue_re_write
            };

            fs::write(path(&self.queue, &self.path), queue_re_write)
                .expect("error re-writting queue");

            let mut file = File::open(path(strip_newline(&to_receive), &self.path))?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            fs::remove_file(path(strip_newline(&to_receive), &self.path))?;

            Ok(buf)
        } else {
            Ok(Vec::new())
        }
    }
}

impl<T> ResourceTables<dyn Resource> for MqTables<T> where T: Mq + 'static {}

impl Resource for MqFilesystem {
    fn from_url(url: Url) -> Result<Self> {
        let path = url.to_file_path();
        match path {
            Ok(path) => {
                let path = path.to_str().unwrap_or(".").to_string();
                Ok(MqFilesystem::new(path))
            }
            Err(_) => bail!("invalid url: {}", url),
        }
    }
}

/// TODO(Dan): This fxn is used across kv-filesystem and mq-filesystem — we might want to make a utils crate.
/// Return the absolute path for the file corresponding to the given key.
fn path(name: &str, base: &str) -> PathBuf {
    PathBuf::from(base).join(name)
}

fn strip_newline(input: &str) -> &str {
    input
        .strip_prefix("\r\n")
        .or(input.strip_suffix('\n'))
        .unwrap_or(input)
        .strip_suffix("\r\n")
        .or(input.strip_suffix('\n'))
        .unwrap_or(input)
}

fn gen_rand_name() -> String {
    format!(
        "{:?}",
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
    )
    
}

impl From<anyhow::Error> for Error {
    fn from(_: anyhow::Error) -> Self {
        Self::GenericError
    }
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::GenericError
    }
}
