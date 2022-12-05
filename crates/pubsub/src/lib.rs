mod implementors;
pub mod providers;
use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use implementors::{apache_kafka, mosquitto};
use slight_common::{impl_resource, BasicState};

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
/// It holds:
///     - a `pubsub_implementor` `String` — this comes directly from a
///     user's `slightfile` and it is what allows us to dynamically
///     dispatch to a specific implementor's implentation, and
///     - the `slight_state` (of type `BasicState`) that contains common
///     things received from the slight binary (i.e., the `config_type`
///     and the `config_toml_file_path`).
#[derive(Clone, Default)]
pub struct Pubsub {
    store: HashMap<String, PubsubState>,
}

#[derive(Clone, Debug)]
struct PubsubState {
    pub_implementor: PubImplementor,
    sub_implementor: SubImplementor,
}
impl Pubsub {
    pub async fn new(name: &str, capability_store: HashMap<String, BasicState>) -> Self {
        let state = capability_store.get(name).unwrap().clone();

        tracing::log::info!("Opening implementor {}", &state.implementor);

        let p = PubImplementor::new(&state.implementor, &state).await;
        let s = SubImplementor::new(&state.implementor, &state).await;

        let store = capability_store
            .iter()
            .map(|c| {
                (
                    c.0.clone(),
                    PubsubState {
                        pub_implementor: p.clone(),
                        sub_implementor: s.clone(),
                    },
                )
            })
            .collect();

        Self { store }
    }
}

impl_resource!(
    Pubsub,
    pubsub::PubsubTables<Pubsub>,
    PubsubState,
    pubsub::add_to_linker,
    "pubsub".to_string()
);

#[async_trait]
impl pubsub::Pubsub for Pubsub {
    type Pub = PubImplementor;
    type Sub = SubImplementor;

    async fn pub_open(&mut self, name: &str) -> Result<Self::Pub, Error> {
        let inner = self.store.get(name).unwrap().clone();
        Ok(inner.pub_implementor)
    }

    async fn pub_publish(
        &mut self,
        self_: &Self::Pub,
        message: PayloadParam<'_>,
        topic: &str,
    ) -> Result<(), Error> {
        match &self_ {
            PubImplementor::ConfluentApacheKafka(pi) => pi.publish(message, topic)?,
            PubImplementor::Mosquitto(pi) => pi.publish(message, topic).await?,
            _ => panic!("Unknown implementor"),
        };

        Ok(())
    }

    async fn sub_open(&mut self, name: &str) -> Result<Self::Sub, Error> {
        let inner = self.store.get(name).unwrap().clone();
        Ok(inner.sub_implementor)
    }

    async fn sub_receive(&mut self, self_: &Self::Sub) -> Result<Vec<u8>, Error> {
        Ok(match &self_ {
            SubImplementor::ConfluentApacheKafka(si) => si.receive().await?,
            SubImplementor::Mosquitto(si) => si.receive().await?,
            _ => panic!("Unknown implementor"),
        })
    }

    async fn sub_subscribe(&mut self, self_: &Self::Sub, topic: &str) -> Result<(), Error> {
        match &self_ {
            SubImplementor::ConfluentApacheKafka(pi) => pi.subscribe(topic).await?,
            SubImplementor::Mosquitto(pi) => pi.subscribe(topic).await?,
            _ => panic!("Unknown implementor"),
        };

        Ok(())
    }
}

/// This defines the available implementor implementations for the `Pubsub` interface.
///
/// As per its' usage in `PubInner`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone, Default)]
pub enum PubImplementor {
    #[default]
    Empty,
    ConfluentApacheKafka(apache_kafka::Pub),
    Mosquitto(mosquitto::Pub),
}

impl PubImplementor {
    async fn new(pubsub_implementor: &str, slight_state: &BasicState) -> Self {
        match pubsub_implementor {
            "pubsub.confluent_apache_kafka" => {
                Self::ConfluentApacheKafka(apache_kafka::Pub::new(slight_state).await)
            }
            "pubsub.mosquitto" => Self::Mosquitto(mosquitto::Pub::new(slight_state).await),
            p => panic!(
                "failed to match provided name (i.e., '{}') to any known host implementations",
                p
            ),
        }
    }
}

/// This defines the available implementor implementations for the `Pubsub` interface.
///
/// As per its' usage in `PubInner`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone, Default)]
pub enum SubImplementor {
    #[default]
    Empty,
    ConfluentApacheKafka(apache_kafka::Sub),
    Mosquitto(mosquitto::Sub),
}

impl SubImplementor {
    async fn new(pubsub_implementor: &str, slight_state: &BasicState) -> Self {
        match pubsub_implementor {
            "pubsub.confluent_apache_kafka" => {
                Self::ConfluentApacheKafka(apache_kafka::Sub::new(slight_state).await)
            }
            "pubsub.mosquitto" => Self::Mosquitto(mosquitto::Sub::new(slight_state).await),
            p => panic!(
                "failed to match provided name (i.e., '{}') to any known host implementations",
                p
            ),
        }
    }
}
