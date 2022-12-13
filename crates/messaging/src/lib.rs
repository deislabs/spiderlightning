mod implementors;
pub mod providers;
use std::collections::HashMap;

use anyhow::Result;
use async_trait::async_trait;

use implementors::{apache_kafka, mosquitto, filesystem::{self, FilesystemImplementor}, azsbus::{self, AzSbusImplementor}};
use slight_common::{impl_resource, BasicState};

/// It is mandatory to `use <interface>::*` due to `impl_resource!`.
/// That is because `impl_resource!` accesses the `crate`'s
/// `add_to_linker`, and not the `<interface>::add_to_linker` directly.
use messaging::*;
wit_bindgen_wasmtime::export!({paths: ["../../wit/messaging.wit"], async: *});
wit_error_rs::impl_error!(messaging::MessagingError);
wit_error_rs::impl_from!(anyhow::Error, messaging::MessagingError::UnexpectedError);
wit_error_rs::impl_from!(
    std::string::FromUtf8Error,
    messaging::MessagingError::UnexpectedError
);

/// The `Messaging` structure is what will implement the `messaging::Messaging` trait
/// coming from the generated code of off `messaging.wit`.
///
/// It holds:
///     - a `messaging_implementor` `String` — this comes directly from a
///     user's `slightfile` and it is what allows us to dynamically
///     dispatch to a specific implementor's implentation, and
///     - the `slight_state` (of type `BasicState`) that contains common
///     things received from the slight binary (i.e., the `config_type`
///     and the `config_toml_file_path`).
#[derive(Clone, Default)]
pub struct Messaging {
    store: HashMap<String, MessagingState>,
}

#[derive(Clone, Debug)]
struct MessagingState {
    pub_implementor: PubImplementor,
    sub_implementor: SubImplementor,
}
impl Messaging {
    pub async fn new(name: &str, capability_store: HashMap<String, BasicState>) -> Self {
        let state = capability_store.get(name).unwrap().clone();

        tracing::log::info!("Opening implementor {}", &state.implementor);

        let p = PubImplementor::new(&state.implementor, &state, &name).await;
        let s = SubImplementor::new(&state.implementor, &state, &name).await;

        let store = capability_store
            .iter()
            .map(|c| {
                (
                    c.0.clone(),
                    MessagingState {
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
    Messaging,
    messaging::MessagingTables<Messaging>,
    MessagingState,
    messaging::add_to_linker,
    "messaging".to_string()
);

#[async_trait]
impl messaging::Messaging for Messaging {
    type Pub = PubImplementor;
    type Sub = SubImplementor;

    async fn pub_open(&mut self, name: &str) -> Result<Self::Pub, MessagingError> {
        let inner = self.store.get(name).unwrap().clone();
        Ok(inner.pub_implementor)
    }

    async fn pub_publish(
        &mut self,
        self_: &Self::Pub,
        message: &[u8],
        topic: &str,
    ) -> Result<(), MessagingError> {
        match &self_ {
            PubImplementor::ConfluentApacheKafka(pi) => pi.publish(message, topic)?,
            PubImplementor::Mosquitto(pi) => pi.publish(message, topic).await?,
            PubImplementor::AzSbus(pi) => pi.send(message).await?,
            PubImplementor::Filesystem(pi) => pi.send(message)?,
            _ => panic!("Unknown implementor"),
        };

        Ok(())
    }

    async fn sub_open(&mut self, name: &str) -> Result<Self::Sub, MessagingError> {
        let inner = self.store.get(name).unwrap().clone();
        Ok(inner.sub_implementor)
    }

    async fn sub_receive(
        &mut self,
        self_: &Self::Sub,
        sub_tok: SubscriptionTokenParam<'_>,
    ) -> Result<Vec<u8>, MessagingError> {
        Ok(match &self_ {
            SubImplementor::ConfluentApacheKafka(pi) => pi.receive(sub_tok).await?,
            SubImplementor::Mosquitto(pi) => pi.receive(sub_tok).await?,
            SubImplementor::AzSbus(pi) => pi.receive().await?,
            SubImplementor::Filesystem(pi) => pi.receive()?,
            _ => panic!("Unknown implementor"),
        })
    }

    async fn sub_subscribe(
        &mut self,
        self_: &Self::Sub,
        topic: &str,
    ) -> Result<String, MessagingError> {
        Ok(match &self_ {
            SubImplementor::ConfluentApacheKafka(pi) => pi.subscribe(topic).await?,
            SubImplementor::Mosquitto(pi) => pi.subscribe(topic).await?,
            SubImplementor::AzSbus(_) => todo!("azsbus does not support subscriptions yet"),
            SubImplementor::Filesystem(_) => todo!("filesystem does not support subscriptions yet"),
            _ => panic!("Unknown implementor"),
        })
    }
}

/// This defines the available implementor implementations for the `Messaging` interface.
///
/// As per its' usage in `PubInner`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone, Default)]
pub enum PubImplementor {
    #[default]
    Empty,
    ConfluentApacheKafka(apache_kafka::Pub),
    Mosquitto(mosquitto::Pub),
    Filesystem(filesystem::FilesystemImplementor),
    AzSbus(azsbus::AzSbusImplementor),
}

impl PubImplementor {
    async fn new(messaging_implementor: &str, slight_state: &BasicState, name: &str) -> Self {
        match messaging_implementor {
            "messaging.confluent_apache_kafka" => {
                Self::ConfluentApacheKafka(apache_kafka::Pub::new(slight_state).await)
            }
            "messaging.mosquitto" => Self::Mosquitto(mosquitto::Pub::new(slight_state).await),
            "messaging.filesystem" | "mq.filesystem" => {
                Self::Filesystem(FilesystemImplementor::new(name))
            }
            "messaging.azsbus" | "mq.azsbus" => {
                Self::AzSbus(AzSbusImplementor::new(slight_state, name).await)
            }
            p => panic!(
                "failed to match provided name (i.e., '{}') to any known host implementations",
                p
            ),
        }
    }
}

/// This defines the available implementor implementations for the `Messaging` interface.
///
/// As per its' usage in `PubInner`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone, Default)]
pub enum SubImplementor {
    #[default]
    Empty,
    ConfluentApacheKafka(apache_kafka::Sub),
    Mosquitto(mosquitto::Sub),
    Filesystem(filesystem::FilesystemImplementor),
    AzSbus(azsbus::AzSbusImplementor),
}

impl SubImplementor {
    async fn new(messaging_implementor: &str, slight_state: &BasicState, name: &str) -> Self {
        match messaging_implementor {
            "messaging.confluent_apache_kafka" | "pubsub.confluent_apache_kafka" => {
                Self::ConfluentApacheKafka(apache_kafka::Sub::new(slight_state).await)
            }
            "messaging.mosquitto" | "pubsub.mosquitto" => {
                Self::Mosquitto(mosquitto::Sub::new(slight_state).await)
            }
            "messaging.filesystem" | "mq.filesystem" => {
                Self::Filesystem(FilesystemImplementor::new(name))
            }
            "messaging.azsbus" | "mq.azsbus" => {
                Self::AzSbus(AzSbusImplementor::new(slight_state, name).await)
            }
            p => panic!(
                "failed to match provided name (i.e., '{}') to any known host implementations",
                p
            ),
        }
    }
}
