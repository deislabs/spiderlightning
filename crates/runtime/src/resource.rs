use slight_events::{events::EventsTables, Events};
use slight_http::{http::HttpTables, Http};
use slight_kv::{kv::KvTables, Kv};
use slight_lockd::{lockd::LockdTables, Lockd};
use slight_mq::{mq::MqTables, Mq};
use slight_pubsub::{pubsub::PubsubTables, Pubsub};
use slight_runtime_configs::{configs::ConfigsTables, Configs};

pub use crate::RuntimeContext;
use crate::{Builder, Ctx};
use anyhow::Result;
use as_any::Downcast;

use slight_events_api::EventHandlerData;
use slight_http_api::HttpHandlerData;
pub use wasmtime::Linker;

/// Guest data for event handler
/// TODO (Joe): abstract this to a general guest data
pub type EventsData = EventHandlerData;
pub type HttpData = HttpHandlerData;

/// A trait for Linkable resources
pub trait Linkable {
    /// Link the resource to the runtime
    fn add_to_linker(linker: &mut Linker<Ctx>) -> Result<()>;
}

macro_rules! impl_linkable {
    ($resource:ty, $add_to_linker:path, $resource_table:ty, $scheme_name:expr) => {
        impl Linkable for $resource {
            fn add_to_linker(linker: &mut Linker<Ctx>) -> Result<()> {
                $add_to_linker(linker, |ctx| {
                    get_table::<$resource, $resource_table>(ctx, $scheme_name)
                })
                .unwrap();
                Ok(())
            }
        }
    };
}

impl_linkable!(
    Kv,
    slight_kv::kv::add_to_linker,
    KvTables<Kv>,
    "kv".to_string()
);

impl_linkable!(
    Http<Builder>,
    slight_http::http::add_to_linker,
    HttpTables<Http<Builder>>,
    "http".to_string()
);

impl_linkable!(
    Mq,
    slight_mq::mq::add_to_linker,
    MqTables<Mq>,
    "mq".to_string()
);

impl_linkable!(
    Lockd,
    slight_lockd::lockd::add_to_linker,
    LockdTables<Lockd>,
    "lockd".to_string()
);

impl_linkable!(
    Configs,
    slight_runtime_configs::configs::add_to_linker,
    ConfigsTables<Configs>,
    "configs".to_string()
);

impl_linkable!(
    Pubsub,
    slight_pubsub::pubsub::add_to_linker,
    PubsubTables<Pubsub>,
    "pubsub".to_string()
);

impl_linkable!(
    Events<Builder>,
    slight_events::events::add_to_linker,
    EventsTables<Events<Builder>>,
    "events".to_string()
);

/// Dynamically dispatch to respective host resource
fn get_table<T, TTable>(cx: &mut Ctx, resource_key: String) -> (&mut T, &mut TTable)
where
    T: 'static,
    TTable: 'static,
{
    let data = cx
        .slight
        .get_mut()
        .get_mut(&resource_key)
        .expect("internal error: Runtime context data is None");
    (
        data.0.as_mut().downcast_mut().unwrap_or_else(|| {
            panic!(
                "internal error: context has key {} but can't be downcast to resource {}",
                &resource_key,
                std::any::type_name::<T>()
            )
        }),
        data.1
            .as_mut()
            .unwrap_or_else(|| {
                panic!(
                    "internal error: table {} is not initialized",
                    std::any::type_name::<TTable>()
                )
            })
            .as_mut()
            .downcast_mut()
            .unwrap_or_else(|| {
                panic!(
                    "internal error: context has key {} but can't be downcast to resource_table {}",
                    &resource_key,
                    std::any::type_name::<TTable>()
                )
            }),
    )
}
