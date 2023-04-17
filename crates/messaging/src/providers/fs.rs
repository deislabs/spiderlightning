use std::fs::{read_dir, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{env, fs};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Pubsub {
    locale: LocaleDir,
}

#[derive(Debug, Clone)]
struct LocaleDir {
    inner: PathBuf,
}

impl LocaleDir {
    fn new(name: &str) -> Self {
        let locale = env::temp_dir().join(name);
        let _ = std::fs::create_dir_all(&locale);

        Self { inner: locale }
    }
}

impl AsRef<PathBuf> for LocaleDir {
    fn as_ref(&self) -> &PathBuf {
        &self.inner
    }
}

impl AsMut<PathBuf> for LocaleDir {
    fn as_mut(&mut self) -> &mut PathBuf {
        &mut self.inner
    }
}

impl Drop for LocaleDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.inner);
    }
}

impl Pubsub {
    pub fn open(name: &str) -> Result<Pubsub> {
        let locale = LocaleDir::new(name);
        Ok(Pubsub { locale })
    }

    pub fn publish(&self, message: &[u8], topic: &str) -> Result<()> {
        let fixed_topic = topic.replace("-", "_");
        let topic_path = self.locale.as_ref().join(fixed_topic.clone());

        // If the topic file doesn't exist, create it
        if !topic_path.exists() {
            fs::File::create(&topic_path)?;
        }

        // Open the topic file for writing and append the message
        let mut topic_file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&topic_path)?;

        // Add a newline character at the end of the message
        let mut message_to_write = Vec::from(message);
        message_to_write.push(b'\n');

        topic_file.write_all(&message_to_write)?;

        // Relay the message to each subscriber
        for entry in fs::read_dir(self.locale.as_ref())? {
            let entry = entry?;
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();

            // If the file is a subscriber file (i.e., "<topic-name>-<subscription-token>")
            if file_name.starts_with(&fixed_topic) && file_name.contains("+") {
                let mut sub_file = OpenOptions::new().write(true).append(true).open(&path)?;

                // Add a newline character at the end of the message
                let mut message_to_write = Vec::from(message);
                message_to_write.push(b'\n');

                sub_file.write_all(&message_to_write)?;
            }
        }

        Ok(())
    }

    pub fn receive(&self, sub_tok: &str) -> Result<Vec<u8>> {
        // Find the subscription file that matches the given subscription token
        let folder = self.locale.as_ref();
        let files = read_dir(folder).expect("Unable to read topics in pubsub");
        tracing::debug!("looking for subscription under: {:?}", files);
        let mut sub_file_name = "".to_string();

        for file in files {
            let path = file.expect("Unable to topic").path();
            let file_name = path.file_name().unwrap().to_str().unwrap().to_string();
            if file_name.contains(sub_tok) {
                sub_file_name = file_name.clone();
                break;
            }
        }
        let sub_file_path = self.locale.as_ref().join(sub_file_name);
        tracing::debug!("found subscription file: {:?}", sub_file_path);

        // Read the entire contents of the subscription file
        let mut sub_file = fs::File::open(&sub_file_path)
            .with_context(|| "no subscription found per given token")?;
        let mut sub_file_contents = Vec::new();
        sub_file.read_to_end(&mut sub_file_contents)?;

        // remove \n (i.e., last element of sub_file_contents)
        sub_file_contents.pop();

        tracing::debug!("subscription file contents: {:?}", sub_file_contents);

        // Read the last message from the subscription file
        let last_message = sub_file_contents.split(|b| *b == b'\n').last();

        // Check if there is a last message
        if last_message.is_none() {
            return Ok(Vec::new());
        }

        let mut last_message = last_message.unwrap().to_vec();
        tracing::debug!("last message: {:?}", last_message);

        sub_file_contents.push(b'\n');
        last_message.push(b'\n');

        // Truncate the subscription file to remove the last message
        let sub_file = fs::OpenOptions::new().write(true).open(&sub_file_path)?;
        sub_file.set_len((sub_file_contents.len() - last_message.len()) as u64)?;
        tracing::debug!("truncated subscription file");

        Ok(Vec::from(last_message))
    }

    pub fn subscribe(&self, topic: &str) -> Result<String> {
        let fixed_topic = topic.replace("-", "_");

        // Generate a random UUID to use as the subscription token
        let sub_token = uuid::Uuid::new_v4().to_string();

        // Create the path for the subscription file
        let sub_file_path = self
            .locale
            .as_ref()
            .join(format!("{}+{}", fixed_topic, sub_token));

        let topic_path = self.locale.as_ref().join(fixed_topic);

        // If the topic file doesn't exist, create it
        if !topic_path.exists() {
            fs::File::create(&topic_path)?;
        }

        // Clone the topic file to the subscription file
        std::fs::copy(topic_path, &sub_file_path)?;

        tracing::debug!("created subscription file: {:?}", sub_file_path);

        Ok(sub_token)
    }
}
