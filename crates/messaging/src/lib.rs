mod implementors;
pub mod providers;
use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;

use implementors::{PubImplementor, SubImplementor, *};
use slight_common::{impl_resource, BasicState};
use slight_file::capability_store::CapabilityStore;
use slight_file::resource::MessagingResource::*;
use slight_file::Resource;

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
pub struct PubInner {
    pub_implementor: Arc<dyn PubImplementor + Send + Sync>,
}

impl PubInner {
    async fn new(
        messaging_implementor: MessagingImplementors,
        slight_state: &BasicState,
        name: &str,
    ) -> Result<Self> {
        Ok(Self {
            pub_implementor: match messaging_implementor {
                #[cfg(feature = "filesystem")]
                MessagingImplementors::Filesystem => {
                    Arc::new(filesystem::FilesystemImplementor::new(name))
                }
                #[cfg(feature = "mosquitto")]
                MessagingImplementors::Mosquitto => {
                    Arc::new(mosquitto::Pub::new(slight_state).await)
                }
                #[cfg(feature = "apache_kafka")]
                MessagingImplementors::ConfluentApacheKafka => {
                    Arc::new(apache_kafka::Pub::new(slight_state).await)
                }
                #[cfg(feature = "azsbus")]
                MessagingImplementors::AzSbus => {
                    Arc::new(azsbus::AzSbusImplementor::new(slight_state).await)
                }
                #[cfg(feature = "natsio")]
                MessagingImplementors::Nats => {
                    Arc::new(natsio::NatsIoImplementor::new(slight_state).await)
                }
            },
        })
    }
}

#[derive(Clone, Debug)]
pub struct SubInner {
    sub_implementor: Arc<dyn SubImplementor + Send + Sync>,
}

impl SubInner {
    async fn new(
        messaging_implementor: MessagingImplementors,
        slight_state: &BasicState,
        name: &str,
    ) -> Result<Self> {
        let sub_implementor = Self {
            sub_implementor: match messaging_implementor {
                #[cfg(feature = "filesystem")]
                MessagingImplementors::Filesystem => {
                    Arc::new(filesystem::FilesystemImplementor::new(name))
                }
                #[cfg(feature = "mosquitto")]
                MessagingImplementors::Mosquitto => {
                    Arc::new(mosquitto::Sub::new(slight_state).await)
                }
                #[cfg(feature = "apache_kafka")]
                MessagingImplementors::ConfluentApacheKafka => {
                    Arc::new(apache_kafka::Sub::new(slight_state).await)
                }
                #[cfg(feature = "azsbus")]
                MessagingImplementors::AzSbus => {
                    Arc::new(azsbus::AzSbusImplementor::new(slight_state).await)
                }
                #[cfg(feature = "natsio")]
                MessagingImplementors::Nats => {
                    Arc::new(natsio::NatsIoImplementor::new(slight_state).await)
                }
            },
        };

        Ok(sub_implementor)
    }
}

#[derive(Clone, Debug)]
struct MessagingState {
    pub_implementor: PubInner,
    sub_implementor: SubInner,
}

impl Messaging {
    pub async fn new(name: &str, capability_store: CapabilityStore<BasicState>) -> Result<Self> {
        let state = capability_store.get(name, "messaging").unwrap().clone();

        tracing::log::info!("Opening implementor {}", &state.implementor);

        let p = PubInner::new(state.implementor.into(), &state, name).await?;
        let s = SubInner::new(state.implementor.into(), &state, name).await?;

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

        Ok(Self { store })
    }
}

impl_resource!(
    Messaging,
    messaging::MessagingTables<Messaging>,
    messaging::add_to_linker,
    "messaging".to_string()
);

#[async_trait]
impl messaging::Messaging for Messaging {
    type Pub = PubInner;
    type Sub = SubInner;

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
        self_.pub_implementor.publish(message, topic).await?;
        Ok(())
    }

    async fn sub_open(&mut self, name: &str) -> Result<Self::Sub, MessagingError> {
        let inner = self.store.get(name).unwrap().clone();
        Ok(inner.sub_implementor)
    }

    async fn sub_subscribe(
        &mut self,
        self_: &Self::Sub,
        topic: &str,
    ) -> Result<String, MessagingError> {
        Ok(self_.sub_implementor.subscribe(topic).await?)
    }

    async fn sub_receive(
        &mut self,
        self_: &Self::Sub,
        sub_tok: SubscriptionTokenParam<'_>,
    ) -> Result<Vec<u8>, MessagingError> {
        Ok(self_.sub_implementor.receive(sub_tok).await?)
    }
}

/// This defines the available implementor implementations for the `Messaging` interface.
///
/// As per its' usage in `PubInner`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
pub enum MessagingImplementors {
    #[cfg(feature = "apache_kafka")]
    ConfluentApacheKafka,
    #[cfg(feature = "mosquitto")]
    Mosquitto,
    #[cfg(feature = "filesystem")]
    Filesystem,
    #[cfg(feature = "azsbus")]
    AzSbus,
    #[cfg(feature = "natsio")]
    Nats,
}

impl From<Resource> for MessagingImplementors {
    fn from(s: Resource) -> Self {
        match s {
            #[cfg(feature = "apache_kafka")]
            Resource::Messaging(ConfluentApacheKafka) => Self::ConfluentApacheKafka,
            #[cfg(feature = "mosquitto")]
            Resource::Messaging(Mosquitto) => Self::Mosquitto,
            #[cfg(feature = "filesystem")]
            Resource::Messaging(Filesystem) | Resource::Messaging(V1Filesystem) => Self::Filesystem,
            #[cfg(feature = "azsbus")]
            Resource::Messaging(Azsbus) | Resource::Messaging(V1Azsbus) => Self::AzSbus,
            #[cfg(feature = "natsio")]
            Resource::Messaging(Nats) => Self::Nats,
            p => panic!(
                "failed to match provided name (i.e., '{p}') to any known host implementations"
            ),
        }
    }
}
