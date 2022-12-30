use anyhow::Result;
use async_trait::async_trait;
use filesystem_pubsub::Pubsub;

use crate::PubImplementor;

use super::SubImplementor;

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
}

#[async_trait]
impl PubImplementor for FilesystemImplementor {
    async fn publish(&self, msg: &[u8], topic: &str) -> Result<()> {
        self.pubsub.publish(msg, topic)
    }
}

#[async_trait]
impl SubImplementor for FilesystemImplementor {
    async fn subscribe(&self, topic: &str) -> Result<String> {
        self.pubsub.subscribe(topic)
    }

    async fn receive(&self, sub_tok: &str) -> Result<Vec<u8>> {
        self.pubsub.receive(sub_tok)
    }
}
