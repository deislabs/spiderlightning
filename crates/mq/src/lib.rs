pub use mq::add_to_linker;
use mq::*;

wit_bindgen_wasmtime::export!("../../wit/mq.wit");

/// A Filesystem implementation for mq interface.
#[derive(Default)]
pub struct MessageQueue {
    queue: Vec<PayloadResult>,
}

impl MessageQueue {
    /// Create a new KvFilesystem.
    pub fn new() -> Self {
        Self { queue: vec![] }
    }
}

impl mq::Mq for MessageQueue {
    type ResourceDescriptor = u64;

    fn get_mq(&mut self) -> Result<Self::ResourceDescriptor, Error> {
        Ok(0)
    }

    fn send(&mut self, rd: &Self::ResourceDescriptor, msg: PayloadParam<'_>) -> Result<(), Error> {
        if *rd != 0 {
            return Err(Error::GenericError);
        }

        self.queue.push(msg.to_vec());

        Ok(())
    }

    fn receive(&mut self, rd: &Self::ResourceDescriptor) -> Result<PayloadResult, Error> {
        if *rd != 0 {
            return Err(Error::GenericError);
        }

        if !self.queue.is_empty() {
            Ok(self.queue.remove(0))
        } else {
            Ok(Vec::new())
        }
    }
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
