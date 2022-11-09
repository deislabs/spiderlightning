use crate::Ctx;
pub use crate::RuntimeContext;

use as_any::Downcast;

use slight_events_api::EventHandlerData;
use slight_http_api::HttpHandlerData;
pub use wasmtime::Linker;

/// Guest data for event handler
/// TODO (Joe): abstract this to a general guest data
pub type EventsData = EventHandlerData;
pub type HttpData = HttpHandlerData;

/// Dynamically dispatch to respective host resource
pub(crate) fn get_host_state<T, TTable>(cx: &mut Ctx, resource_key: String) -> (&mut T, &mut TTable)
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
