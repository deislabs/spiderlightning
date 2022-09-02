# Service Implementation 101
> valid for: 2022-09-02

Are you hoping to learn how to implement a new service for our existing capabilities (i.e., kv, lockd, mq, or pubsub)? You have come to the right place!

> Note: If you would like to develop a new capability outside of our provided interfaces, you will first have to create its' WIT. Please, create an issue to start a design discussion, and make a PR with the `.wit` file itself prior to its' `slight` implementation.

The DeisLabs Engineering Team has designed each of its' main capabilities to be easily extensible, and some macros to take care of boiler-plate code for you. In this tutorial, we will implement a new `pubsub` implementation — a local one with Mosquitto!

## Getting Started

Assuming that you are on the root of the SpiderLightning repository that you locally cloned, to get started, do:

```sh
git branch my_local_pubsub_implementation c73826a1d522bc3e1d2f0387481880edecd3e3d3
```

## Adding a New Dependency

To ease our development, will be making use of the [mosquitto-rs](https://crates.io/crates/mosquitto-rs) crate, and a couple of others.

At the end of our `Cargo.toml` in the `crates/pubsub` directory, add:

```toml
# pubsub.mosquitto deps
mosquitto-rs = { version = "0.4.0", features = ["default", "vendored-openssl"]}
futures = "0.3"
async-channel = "1.5"
```

Next up, inside `lib.rs` in the `crates/pubsub/src` directory, we'll have to create a new variant for the `PubImplementor` and `SubImplementor` enums. Like so:

```rs
#[derive(Debug, Clone)]
enum PubImplementor {
    ConfluentApacheKafka(PubConfluentApacheKafkaImplementor),
    Local(MosquittoImplementor)
};

// [..]

#[derive(Debug, Clone)]
enum SubImplementor {
    ConfluentApacheKafka(SubConfluentApacheKafkaImplementor),
    Local(MosquittoImplementor)
}
```

> Note: Our `Local` variants for both `enum`s hold a struct called `MosquittoImplementor` but we haven't implemented that yet.

With that done, we now have to take care of the `new` function for both `PubImplementor`, and `SubImplementor`. Considering that the `new` function is what is responsible for mapping a string from a user's slightfile to a specific implementor, this is the time where we decide what string should refer to our local implementation — let's say `pubsub.mosquitto`. Now, we can go ahead and change the `new` functions, like so:

```rs
impl PubImplementor {
    fn new(pubsub_implementor: &str, slight_state: &BasicState) -> Self {
        match pubsub_implementor {
            "pubsub.confluent_apache_kafka" => {
                Self::ConfluentApacheKafka(PubConfluentApacheKafkaImplementor::new(slight_state))
            },
            "pubsub.mosquitto" => {
                Self::Local(MosquittoImplementor::new(slight_state))
            },
            p => panic!(
                "failed to match provided name (i.e., '{}') to any known host implementations",
                p
            ),
        }
    }
}

// [..]

impl SubImplementor {
    fn new(pubsub_implementor: &str, slight_state: &BasicState) -> Self {
        match pubsub_implementor {
            "pubsub.confluent_apache_kafka" => {
                Self::ConfluentApacheKafka(SubConfluentApacheKafkaImplementor::new(slight_state))
            },
            "pubsub.local" => {
                Self::Local(MosquittoImplementor::new(slight_state))
            },
            p => panic!(
                "failed to match provided name (i.e., '{}') to any known host implementations",
                p
            ),
        }
    }
}
```

> Note: a couple of `use` statements were ommited.

Once the we get a `&str` match for `pubsub.mosquitto`, the `new` function returns the `Local` variant of the `enum` having it populated with a new instance of the `MosquittoImplementor` — let's work on making these missing types now.

Inside of the `implementors/` folder, let's create a new file called `mosquitto.rs` — this is the file that will contain our `MosquittoImplementor`. After that, inside `implementors/mod.rs`, let's add:
```rs
pub mod mosquitto
```

Next, back in `implementors/mosquitto.rs` Here's what each of them will need:

```rust
use anyhow::Result;
use slight_common::BasicState;

#[derive(Clone)]
pub struct MosquittoImplementor {
    // -snip-
}

impl std::fmt::Debug for MosquittoImplementor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MosquittoImplementor")
    }
}

// Pub+Sub
impl MosquittoImplementor  {
    pub fn new(slight_state: &BasicState) -> Self {
        // -snip-
    }
}

// Pub
impl MosquittoImplementor  {
    pub fn send_message_to_topic(
        &self,
        msg_key: &[u8],
        msg_value: &[u8],
        topic: &str,
    ) -> Result<()> {
        // -snip-
    }
}

// Sub
impl MosquittoImplementor {
    pub fn subscribe_to_topic(&self, topic: Vec<&str>) -> Result<()> {
        // -snip-
    }

    pub fn poll_for_message(&self, timeout_in_secs: u64) -> Result<KafkaMessage> {
        // -snip-
    }
}
```

This sets the layout of all the functions we need to be able to dynamically dispatch to from `lib.rs`.

Now, everything inside these function is specific to the implementation we are working on, so not too relevant to this tutorial specifically, but, if you are following along, in the end, your `mosquitto.rs` file should look like this:

```rs
use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use async_channel::Receiver;
use futures::executor::block_on;
use mosquitto_rs::{Client, Message, QoS};
use slight_common::BasicState;

#[derive(Clone)]
pub struct MosquittoImplementor {
    mqtt: Arc<Mutex<Client>>,
    host: String,
    port: i32,
    subscriptions: Arc<Mutex<Vec<String>>>,
    subscriber: Arc<Mutex<Option<Receiver<Message>>>>,
}

impl std::fmt::Debug for MosquittoImplementor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MosquittoImplementor")
    }
}

// Pub+Sub
impl MosquittoImplementor {
    pub fn new(slight_state: &BasicState) -> Self {
        let mqtt = Client::with_auto_id().unwrap();
        let host = String::from_utf8(
            slight_runtime_configs::get(
                &slight_state.secret_store,
                "MOSQUITTO_HOST",
                &slight_state.slightfile_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'MOSQUITTO_HOST' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap();

        let port = String::from_utf8(
            slight_runtime_configs::get(
                &slight_state.secret_store,
                "MOSQUITTO_PORT",
                &slight_state.slightfile_path,
            )
            .with_context(|| {
                format!(
                    "failed to get 'MOSQUITTO_PORT' secret using secret store type: {}",
                    slight_state.secret_store
                )
            })
            .unwrap(),
        )
        .unwrap()
        .parse::<i32>()
        .unwrap();

        Self {
            mqtt: Arc::new(Mutex::new(mqtt)),
            host,
            port,
            subscriptions: Arc::new(Mutex::new(Vec::new())),
            subscriber: Arc::new(Mutex::new(None)),
        }
        // ^^^ arbitrarily chosing to create with 5 threads
        // (threads run notification functions)
    }
}

// Pub
impl MosquittoImplementor {
    pub fn send_message_to_topic(
        &self,
        msg_key: &[u8],
        msg_value: &[u8],
        topic: &str,
    ) -> Result<()> {
        let formatted_message_with_key = &format!(
            "{}-{}",
            std::str::from_utf8(msg_key)?,
            std::str::from_utf8(msg_value)?
        );
        // ^^^ arbitrarily formatting msg key and value like
        // (as we have more implementors for pubsub, we should consider if we even
        // want a key in the pubsub implementation)

        block_on(self.mqtt.lock().as_mut().unwrap().connect(
            &self.host,
            self.port,
            std::time::Duration::from_secs(5),
            None,
        ))?;

        block_on(self.mqtt.lock().as_mut().unwrap().publish(
            topic,
            formatted_message_with_key.as_bytes(),
            QoS::AtMostOnce,
            false,
        ))?;
        Ok(())
    }
}

// Sub
impl MosquittoImplementor {
    pub fn subscribe_to_topic(&self, topic: Vec<String>) -> Result<()> {
        for t in topic {
            self.subscriptions.lock().unwrap().push(t);
        }

        *self.subscriber.lock().unwrap() = self.mqtt.lock().as_mut().unwrap().subscriber();
        Ok(())
    }

    pub fn poll_for_message(&self, _: u64) -> Result<String> {
        // ^^^ timeout unused here, this probably hints it's not something we want in the
        // overall interface
        block_on(self.mqtt.lock().as_mut().unwrap().connect(
            &self.host,
            self.port,
            std::time::Duration::from_secs(5),
            None,
        ))?;

        for t in self.subscriptions.lock().unwrap().iter() {
            block_on(
                self.mqtt
                    .lock()
                    .as_mut()
                    .unwrap()
                    .subscribe(t, QoS::AtMostOnce),
            )?;
        }

        let msg = format!(
            "{:?}",
            String::from_utf8(
                block_on(
                    self.subscriber
                        .lock()
                        .as_mut()
                        .unwrap()
                        .as_mut()
                        .unwrap()
                        .recv()
                )?
                .payload
            )
        );

        Ok(msg)
    }
}
```

We have mentioned dynamic dispatching — Let's do that now (in lib.rs`):
```rs
    fn pub_send_message_to_topic(
        &mut self,
        self_: &Self::Pub,
        msg_key: PayloadParam<'_>,
        msg_value: PayloadParam<'_>,
        topic: &str,
    ) -> Result<(), Error> {
        match &self_.pub_implementor {
            PubImplementor::ConfluentApacheKafka(pi) => {
                pi.send_message_to_topic(msg_key, msg_value, topic)?
            }
            PubImplementor::Local(pi) => pi.send_message_to_topic(msg_key, msg_value, topic)?,
        };

// -snip-

    fn sub_subscribe_to_topic(&mut self, self_: &Self::Sub, topic: Vec<&str>) -> Result<(), Error> {
        match &self_.sub_implementor {
            SubImplementor::ConfluentApacheKafka(si) => si.subscribe_to_topic(topic)?,
            SubImplementor::Local(si) => {
                si.subscribe_to_topic(topic.iter().map(|s| s.to_string()).collect::<Vec<String>>())?
            }
        }

// -snip-
    fn sub_poll_for_message(
        &mut self,
        self_: &Self::Sub,
        timeout_in_secs: u64,
    ) -> Result<Message, Error> {
        Ok(match &self_.sub_implementor {
            SubImplementor::ConfluentApacheKafka(si) => {
                si.poll_for_message(timeout_in_secs)
                    .map(|f| pubsub::Message {
                        key: f.0,
                        value: f.1,
                    })?
            }
            SubImplementor::Local(si) => {
                si.poll_for_message(timeout_in_secs)
                    .map(|f| pubsub::Message {
                        key: Some("batch".as_bytes().to_vec()),
                        value: Some(f.as_bytes().to_vec()),
                    })?
            }
        })
    }
```

In here, all we are doing is adding calls to the functions implemented by the `MosquittoImplementor` under the `Local` variant of the `Pub/SubImplementor` enums, so that we are handling the case where a user provides a `toml` with `pubsub.mosquitto` and makes use of its' functionality.

Next up, you'll have to make a change to the slight runner itself at: `slight/src/commands/run.rs`. That is, instead of:
```rs
const PUBSUB_HOST_IMPLEMENTORS: [&str; 1] = ["pubsub.confluent_apache_kafka"];
```

We want the `PUBSUB_HOST_IMPLEMENTORS` array to be:
```rs
const PUBSUB_HOST_IMPLEMENTORS: [&str; 2] = ["pubsub.confluent_apache_kafka", "pubsub.mosquitto"];
```

Now, you just need to create a slightfile, say, `mosquitto_slightfile.toml`, for your new implementor under their respective example crates — In this case, we want to create the exact same `mosquitto_slightfile.toml` for both the `examples/pubsub-consumer-demo` and `examples/pubsub-producer-demo`. It should look like this:
```toml
specversion = "0.1"
secret_store = "configs.azapp"

[[capability]]
name = "pubsub.mosquitto"
```

To finish off, make sure to add your example in the list of example runs under the `run-rust` Makefile rule — for our new pubsub implementor, it will look like so:
```Makefile
    RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/pubsub-consumer-demo/mosquitto_slightfile.toml' run -m ./examples/pubsub-consumer-demo/target/wasm32-wasi/release/pubsub-consumer-demo.wasm &
	RUST_LOG=$(LOG_LEVEL) $(SLIGHT) -c './examples/pubsub-producer-demo/mosquitto_slightfile.toml' run -m ./examples/pubsub-producer-demo/target/wasm32-wasi/release/pubsub-producer-demo.wasm
```

With this, you should have the basic understanding on how to get started developing capabilities for Slight — Now, all you are missing is just the implementation of the service itself! We are looking forward to your contributions 🚀