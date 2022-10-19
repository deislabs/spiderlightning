mod implementors;
pub mod providers;
use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use implementors::{
    apache_kafka::{PubConfluentApacheKafkaImplementor, SubConfluentApacheKafkaImplementor},
    mosquitto::MosquittoImplementor,
};
use slight_common::{impl_resource, BasicState};
use uuid::Uuid;

/// It is mandatory to `use <interface>::*` due to `impl_resource!`.
/// That is because `impl_resource!` accesses the `crate`'s
/// `add_to_linker`, and not the `<interface>::add_to_linker` directly.
use pubsub::*;
wit_bindgen_wasmtime::export!({paths: ["../../wit/pubsub.wit"], async: *});
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

impl_resource!(Pubsub, pubsub::PubsubTables<Pubsub>, PubsubState);

/// This is the type of the `host_state` property from our `Pubsub` structure.
///
/// It holds:
///     - a `pubsub_implementor` `String` — this comes directly from a
///     user's `slightfile` and it is what allows us to dynamically
///     dispatch to a specific implementor's implentation, and
///     - the `slight_state` (of type `BasicState`) that contains common
///     things received from the slight binary (i.e., the `resource_map`,
///     the `config_type`, and the `config_toml_file_path`).
#[derive(Clone, Default)]
pub struct PubsubState {
    implementor: String,
    capability_store: HashMap<String, BasicState>,
}

impl PubsubState {
    pub fn new(implementor: String, capability_store: HashMap<String, BasicState>) -> Self {
        Self {
            implementor,
            capability_store,
        }
    }
}

#[async_trait]
impl pubsub::Pubsub for Pubsub {
    type Pub = PubInner;
    type Sub = SubInner;

    async fn pub_open(&mut self, name: &str) -> Result<Self::Pub, Error> {
        // populate our inner pubsub object w/ the state received from `slight`
        // (i.e., what type of pubsub implementor we are using), and the assigned
        // name of the object.
        let state = self.host_state.capability_store.get(name).unwrap().clone();

        tracing::log::info!("Opening implementor {}", &state.implementor);

        let inner = Self::Pub::new(&state.implementor, &state).await;

        state
            .resource_map
            .lock()
            .unwrap()
            .set(inner.resource_descriptor.clone(), Box::new(inner.clone()));

        Ok(inner)
    }

    async fn sub_open(&mut self, name: &str) -> Result<Self::Sub, Error> {
        // populate our inner pubsub object w/ the state received from `slight`
        // (i.e., what type of pubsub implementor we are using), and the assigned
        // name of the object.
        let state = if let Some(r) = self.host_state.capability_store.get(name) {
            r.clone()
        } else {
            if let Some(r) = self
                .host_state
                .capability_store
                .get(&self.host_state.implementor)
            {
                r.clone()
            } else {
                panic!(
                    "could not find capability under name '{}' for implementor '{}'",
                    name, &self.host_state.implementor
                );
            }
        };

        tracing::log::info!("Opening implementor {}", &state.implementor);

        let inner = Self::Sub::new(&state.implementor, &state).await;

        state
            .resource_map
            .lock()
            .unwrap()
            .set(inner.resource_descriptor.clone(), Box::new(inner.clone()));

        Ok(inner)
    }

    async fn pub_publish(
        &mut self,
        self_: &Self::Pub,
        message: PayloadParam<'_>,
        topic: &str,
    ) -> Result<(), Error> {
        match &self_.pub_implementor {
            PubImplementor::ConfluentApacheKafka(pi) => pi.publish(message, topic)?,
            PubImplementor::Mosquitto(pi) => pi.publish(message, topic).await?,
        };

        Ok(())
    }

    async fn sub_subscribe(&mut self, self_: &Self::Sub, topic: &str) -> Result<(), Error> {
        match &self_.sub_implementor {
            SubImplementor::ConfluentApacheKafka(si) => si.subscribe(topic)?,
            SubImplementor::Mosquitto(si) => si.subscribe(topic)?,
        }

        Ok(())
    }

    async fn sub_receive(&mut self, self_: &Self::Sub) -> Result<Vec<u8>, Error> {
        Ok(match &self_.sub_implementor {
            SubImplementor::ConfluentApacheKafka(si) => si.receive().await?,
            SubImplementor::Mosquitto(si) => si.receive().await?,
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

impl slight_events_api::Watch for PubInner {}

impl PubInner {
    async fn new(pub_implementor: &str, slight_state: &BasicState) -> Self {
        Self {
            pub_implementor: PubImplementor::new(pub_implementor, slight_state).await,
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

impl slight_events_api::Watch for SubInner {}

impl SubInner {
    async fn new(sub_implementor: &str, slight_state: &BasicState) -> Self {
        Self {
            sub_implementor: SubImplementor::new(sub_implementor, slight_state).await,
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
    Mosquitto(MosquittoImplementor),
}

impl PubImplementor {
    async fn new(pubsub_implementor: &str, slight_state: &BasicState) -> Self {
        match pubsub_implementor {
            "pubsub.confluent_apache_kafka" => Self::ConfluentApacheKafka(
                PubConfluentApacheKafkaImplementor::new(slight_state).await,
            ),
            "pubsub.mosquitto" => Self::Mosquitto(MosquittoImplementor::new(slight_state).await),
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
    Mosquitto(MosquittoImplementor),
}

impl SubImplementor {
    async fn new(pubsub_implementor: &str, slight_state: &BasicState) -> Self {
        match pubsub_implementor {
            "pubsub.confluent_apache_kafka" => Self::ConfluentApacheKafka(
                SubConfluentApacheKafkaImplementor::new(slight_state).await,
            ),
            "pubsub.mosquitto" => Self::Mosquitto(MosquittoImplementor::new(slight_state).await),
            p => panic!(
                "failed to match provided name (i.e., '{}') to any known host implementations",
                p
            ),
        }
    }
}
