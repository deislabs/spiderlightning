use std::{
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;

/// This is the underlying struct behind the `Filesystem` variant of the `MqImplementor` enum.
///
/// It provides one property that pertains solely to the filesystem implementation of
/// of this capability:
///     - `base`.
///
/// As per its' usage in `MqImplementor`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
pub struct FilesystemImplementor {
    /// The base path for where the message queue can be found in your file-system
    base: String,
    /// The name of a hidden file that maintains the queue order and
    /// contains the names of files representating queue elements
    queue: String,
}

impl FilesystemImplementor {
    pub fn new(name: &str) -> Self {
        Self {
            base: Path::new("/tmp").join(name).to_str().unwrap().to_owned(),
            queue: ".queue".to_string(),
        }
    }

    pub fn send(&self, msg: &[u8]) -> Result<()> {
        // get a random name for a queue element
        let rand_file_name = format!(
            "{:?}",
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
        );

        fs::create_dir_all(&self.base)?;

        // create a file to store the queue element data
        let mut file = File::create(PathBuf::from(&self.base).join(&rand_file_name))?;

        // write to file msg sent
        file.write_all(msg)?;

        // open/create queue and store one random name for a queue element per line
        let mut queue = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(PathBuf::from(&self.base).join(&self.queue))?;

        // add queue element name to the bottom of the queue
        writeln!(queue, "{}", rand_file_name)?;

        Ok(())
    }

    pub fn receive(&self) -> Result<Vec<u8>> {
        fs::create_dir_all(&self.base)?;

        // get the queue
        let queue = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(PathBuf::from(&self.base).join(&self.queue))?;

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
            fs::write(
                PathBuf::from(&self.base).join(&self.queue),
                queue_post_receive,
            )?;

            // remove \n char from end of queue element
            to_receive.pop();

            // get element at top of queue
            let mut file = File::open(PathBuf::from(&self.base).join(&to_receive))?;
            let mut buf = Vec::new();

            // read element's message
            file.read_to_end(&mut buf)?;

            // clean-up element from disk
            fs::remove_file(PathBuf::from(&self.base).join(&to_receive))?;

            Ok(buf)
        } else {
            // if queue is empty, respond with empty string
            Ok(Vec::new())
        }
    }
}
