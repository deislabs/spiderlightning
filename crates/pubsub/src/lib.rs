mod implementors;
pub mod providers;

/// The `SCHEME_NAME` defines the name under which a resource is
/// identifiable by in a `ResourceMap`.
const SCHEME_NAME: &str = "pubsub";



use anyhow::Result;


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
    type Pub = PubInner;
    type Sub = SubInner;

    fn pub_open(&mut self) -> Result<Self::Pub, Error> {
        // populate our inner pubsub object w/ the state received from `slight`
        // (i.e., what type of pubsub implementor we are using), and the assigned
        // name of the object.
        let inner = Self::Pub::new(
            &self.host_state.pubsub_implementor,
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

    fn sub_open(&mut self) -> Result<Self::Sub, Error> {
        // populate our inner pubsub object w/ the state received from `slight`
        // (i.e., what type of pubsub implementor we are using), and the assigned
        // name of the object.
        let inner = Self::Sub::new(
            &self.host_state.pubsub_implementor,
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
        };

        Ok(())
    }

    fn sub_subscribe_to_topic(&mut self, self_: &Self::Sub, topic: Vec<&str>) -> Result<(), Error> {
        match &self_.sub_implementor {
            SubImplementor::ConfluentApacheKafka(si) => si.subscribe_to_topic(topic)?,
        }

        Ok(())
    }

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
        })
    }
}

/// This is the type of the associated type coming from the `pubsub::Pubsub` trait implementation.
///
/// It holds:
///     - a `pub_implementor` (i.e., a variant `PubImplementor` `enum`), and
///     - a `resource_descriptor` (i.e., an UUID that uniquely identifies
///     resource's instance).
///
/// It must `derive`:
///     - `Debug` due to a constraint on the associated type.
///     - `Clone` because the `ResourceMap` it will be added onto,
///     must own its' data.
///
/// It must be public because the implementation of `pubsub::Pubsub` cannot leak
/// a private type.
#[derive(Debug, Clone)]
pub struct PubInner {
    pub_implementor: PubImplementor,
    resource_descriptor: String,
}

impl runtime::resource::Watch for PubInner {}

impl PubInner {
    fn new(pub_implementor: &str, slight_state: &BasicState) -> Self {
        Self {
            pub_implementor: PubImplementor::new(pub_implementor, slight_state),
            resource_descriptor: Uuid::new_v4().to_string(),
        }
    }
}

/// This is the type of the associated type coming from the `pubsub::Pubsub` trait implementation.
///
/// It holds:
///     - a `sub_implementor` (i.e., a variant `SubImplementor` `enum`), and
///     - a `resource_descriptor` (i.e., an UUID that uniquely identifies
///     resource's instance).
///
/// It must `derive`:
///     - `Debug` due to a constraint on the associated type.
///     - `Clone` because the `ResourceMap` it will be added onto,
///     must own its' data.
///
/// It must be public because the implementation of `pubsub::Pubsub` cannot leak
/// a private type.
#[derive(Debug, Clone)]
pub struct SubInner {
    sub_implementor: SubImplementor,
    resource_descriptor: String,
}

impl runtime::resource::Watch for SubInner {}

impl SubInner {
    fn new(sub_implementor: &str, slight_state: &BasicState) -> Self {
        Self {
            sub_implementor: SubImplementor::new(sub_implementor, slight_state),
            resource_descriptor: Uuid::new_v4().to_string(),
        }
    }
}

/// This defines the available implementor implementations for the `Pubsub` interface.
///
/// As per its' usage in `PubInner`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
enum PubImplementor {
    ConfluentApacheKafka(PubConfluentApacheKafkaImplementor),
}

impl PubImplementor {
    fn new(pubsub_implementor: &str, slight_state: &BasicState) -> Self {
        match pubsub_implementor {
            "pubsub.confluent_apache_kafka" => {
                Self::ConfluentApacheKafka(PubConfluentApacheKafkaImplementor::new(slight_state))
            }
            p => panic!(
                "failed to match provided name (i.e., '{}') to any known host implementations",
                p
            ),
        }
    }
}

/// This defines the available implementor implementations for the `Pubsub` interface.
///
/// As per its' usage in `SubInner`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
enum SubImplementor {
    ConfluentApacheKafka(SubConfluentApacheKafkaImplementor),
}

impl SubImplementor {
    fn new(pubsub_implementor: &str, slight_state: &BasicState) -> Self {
        match pubsub_implementor {
            "pubsub.confluent_apache_kafka" => {
                Self::ConfluentApacheKafka(SubConfluentApacheKafkaImplementor::new(slight_state))
            }
            p => panic!(
                "failed to match provided name (i.e., '{}') to any known host implementations",
                p
            ),
        }
    }
}
