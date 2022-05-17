use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use kv::*;
pub use kv::add_to_linker;

wit_bindgen_wasmtime::export!("../../wit/kv.wit");

#[derive(Default)]
pub struct KV_FS {
    path: String,
}

impl KV_FS {
    pub fn new(path: String) -> Self {
        Self { path }
    }
}

impl kv::Kv for KV_FS {
    type KvRd = u64;

    fn get_kv(&mut self) -> Result<Self::KvRd,Error> {
        Ok(0)
    }
    
    fn get(&mut self,key: & str,rd: & Self::KvRd,) -> Result<PayloadResult,Error> {
        if *rd != 0 {
            return Err(Error::Error);
        }

        let mut file = match File::open(path(&key, &self.path)?) {
            Ok(f) => f,
            Err(_) => return Ok(Vec::new()),
        };

        let mut buf = Vec::new();
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }
    
    fn set(&mut self,key: & str,value: &[u8],rd: & Self::KvRd,) -> Result<(),Error> {
        if *rd != 0 {
            return Err(Error::Error);
        }
        let mut file = File::create(path(&key, &self.path)?)?;
        file.write_all(&value)?;
        Ok(())
    }
}

/// Return the absolute path for the file corresponding to the given key.
fn path(name: &str, base: &str) -> Result<PathBuf, anyhow::Error> {
    Ok(PathBuf::from(base).join(name))
}

impl From<anyhow::Error> for Error {
    fn from(_: anyhow::Error) -> Self {
        Self::Error
    }
}

impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::Error
    }
}