use anyhow::Result;
use filesystem_pubsub::Pubsub;

/// This is the underlying struct behind the `Filesystem` variant of the implementors enum.
#[derive(Debug, Clone)]
pub struct FilesystemImplementor {
    pubsub: Pubsub,
}

impl FilesystemImplementor {
    pub fn new(name: &str) -> Self {
        Self {
            pubsub: Pubsub::open(name).unwrap(),
        }
    }

    pub fn subscribe(&self, topic: &str) -> Result<String> {
        self.pubsub.subscribe(topic)
    }

    pub fn publish(&self, msg: &[u8], topic: &str) -> Result<()> {
        self.pubsub.publish(msg, topic)
    }

    pub fn receive(&self, sub_tok: &str) -> Result<Vec<u8>> {
        self.pubsub.receive(sub_tok)
    }
}
