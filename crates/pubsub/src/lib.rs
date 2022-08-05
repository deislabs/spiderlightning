mod implementors;
pub mod providers;

/// The `SCHEME_NAME` defines the name under which a resource is
/// identifiable by in a `ResourceMap`.
const SCHEME_NAME: &str = "pubsub";

use std::sync::{Arc, Mutex};

use anyhow::Result;
use crossbeam_channel::Sender;
use events_api::Event;
use implementors::apache_kafka::{
    PubConfluentApacheKafkaImplementor, SubConfluentApacheKafkaImplementor,
};
use runtime::{impl_resource, resource::BasicState};
use uuid::Uuid;

/// It is mandatory to `use <interface>::*` due to `impl_resource!`.
/// That is because `impl_resource!` accesses the `crate`'s
/// `add_to_linker`, and not the `<interface>::add_to_linker` directly.
use pubsub::*;
wit_bindgen_wasmtime::export!("../../wit/pubsub.wit");
wit_error_rs::impl_error!(pubsub::Error);
wit_error_rs::impl_from!(anyhow::Error, pubsub::Error::ErrorWithDescription);
wit_error_rs::impl_from!(
    std::string::FromUtf8Error,
    pubsub::Error::ErrorWithDescription
);

/// The `Pubsub` structure is what will implement the `pubsub::Pubsub` trait
/// coming from the generated code of off `pubsub.wit`.
///
/// It maintains a `host_state`.
pub struct Pubsub {
    host_state: PubsubState,
}

impl_resource!(
    Pubsub,
    pubsub::PubsubTables<Pubsub>,
    PubsubState,
    SCHEME_NAME.to_string()
);

/// This is the type of the `host_state` property from our `Pubsub` structure.
///
/// It holds:
///     - a `pubsub_implementor` `String` — this comes directly from a
///     user's `slightfile` and it is what allows us to dynamically
///     dispatch to a specific implementor's implentation, and
///     - the `slight_state` (of type `BasicState`) that contains common
///     things received from the slight binary (i.e., the `resource_map`,
///     the `config_type`, and the `config_toml_file_path`).
pub struct PubsubState {
    pubsub_implementor: String,
    slight_state: BasicState,
}

impl PubsubState {
    pub fn new(pubsub_implementor: String, slight_state: BasicState) -> Self {
        Self {
            pubsub_implementor,
            slight_state,
        }
    }
}

impl pubsub::Pubsub for Pubsub {
    type Pubsub = PubsubInner;

    fn pubsub_open_pub(&mut self) -> Result<Self::Pubsub, Error> {
        // populate our inner pubsub object w/ the state received from `slight`
        // (i.e., what type of pubsub implementor we are using), and the assigned
        // name of the object.
        let inner = Self::Pubsub::new(
            &format!("{}.pub", &self.host_state.pubsub_implementor), // append ".pub" to indicate that's all we are making
            &self.host_state.slight_state,
        );

        self.host_state
            .slight_state
            .resource_map
            .lock()
            .unwrap()
            .set(inner.resource_descriptor.clone(), Box::new(inner.clone()));

        Ok(inner)
    }

    fn pubsub_open_sub(&mut self) -> Result<Self::Pubsub, Error> {
        // populate our inner pubsub object w/ the state received from `slight`
        // (i.e., what type of pubsub implementor we are using), and the assigned
        // name of the object.
        let inner = Self::Pubsub::new(
            &format!("{}.sub", &self.host_state.pubsub_implementor), // append ".sub" to indicate that's all we are making
            &self.host_state.slight_state,
        );

        self.host_state
            .slight_state
            .resource_map
            .lock()
            .unwrap()
            .set(inner.resource_descriptor.clone(), Box::new(inner.clone()));

        Ok(inner)
    }

    fn pubsub_send_message_to_topic(
        &mut self,
        self_: &Self::Pubsub,
        msg_key: PayloadParam<'_>,
        msg_value: PayloadParam<'_>,
        topic: &str,
    ) -> Result<(), Error> {
        match &self_.pubsub_implementor {
            PubsubImplementor::ConfluentApacheKafka(pi, _) => pi
                .as_ref()
                .unwrap()
                .send_message_to_topic(msg_key, msg_value, topic)?,
        };

        Ok(())
    }

    fn pubsub_subscribe_to_topic(
        &mut self,
        self_: &Self::Pubsub,
        topic: Vec<&str>,
    ) -> Result<(), Error> {
        match &self_.pubsub_implementor {
            PubsubImplementor::ConfluentApacheKafka(_, si) => {
                si.as_ref().unwrap().subscribe_to_topic(topic)?
            }
        }

        Ok(())
    }

    fn pubsub_poll_for_message(
        &mut self,
        self_: &Self::Pubsub,
        timeout_in_secs: u64,
    ) -> Result<Message, Error> {
        Ok(match &self_.pubsub_implementor {
            PubsubImplementor::ConfluentApacheKafka(_, si) => si
                .as_ref()
                .unwrap()
                .poll_for_message(timeout_in_secs)
                .map(|f| pubsub::Message {
                    key: f.0,
                    value: f.1,
                })?,
        })
    }
}

/// This is the type of the associated type coming from the `lockd::Lockd` trait
/// implementation.
///
/// It holds:
///     - a `lockd_implementor` (i.e., a variant `LockdImplementor` `enum`), and
///     - a `resource_descriptor` (i.e., an UUID that uniquely identifies
///     resource's instance).
///
/// It must `derive`:
///     - `Debug` due to a constraint on the associated type.
///     - `Clone` because the `ResourceMap` it will be added onto,
///     must own its' data.
///
/// It must be public because the implementation of `lockd::Lockd` cannot leak
/// a private type.
#[derive(Debug, Clone)]
pub struct PubsubInner {
    pubsub_implementor: PubsubImplementor,
    resource_descriptor: String,
}

impl PubsubInner {
    fn new(pubsub_implementor: &str, slight_state: &BasicState) -> Self {
        Self {
            pubsub_implementor: PubsubImplementor::new(pubsub_implementor, slight_state),
            resource_descriptor: Uuid::new_v4().to_string(),
        }
    }
}

impl runtime::resource::Watch for PubsubInner {
    fn watch(&mut self, key: &str, sender: Arc<Mutex<Sender<Event>>>) -> Result<()> {
        todo!(
            "got {} and {:?}, but got nothing to do with it yet",
            key,
            sender
        );
    }
}

/// This defines the available implementor implementations for the `Lockd` interface.
///
/// As per its' usage in `LockdInner`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
enum PubsubImplementor {
    ConfluentApacheKafka(
        Option<PubConfluentApacheKafkaImplementor>,
        Option<SubConfluentApacheKafkaImplementor>,
    ),
}

impl PubsubImplementor {
    fn new(pubsub_implementor: &str, slight_state: &BasicState) -> Self {
        match pubsub_implementor {
            "pubsub.confluent_apache_kafka.pub" => Self::ConfluentApacheKafka(
                Some(PubConfluentApacheKafkaImplementor::new(slight_state)),
                None,
            ),
            "pubsub.confluent_apache_kafka.sub" => Self::ConfluentApacheKafka(
                None,
                Some(SubConfluentApacheKafkaImplementor::new(slight_state)),
            ),
            p => panic!(
                "failed to match provided name (i.e., '{}') to any known host implementations",
                p
            ),
        }
    }
}
