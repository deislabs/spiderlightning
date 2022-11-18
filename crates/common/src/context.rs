use slight_events_api::EventHandlerData;
use slight_http_api::HttpHandlerData;

/// A WebAssembly runtime context to be consumed by the wasm component.
pub trait Ctx {
    /// Get the mutable reference to the http handler data.
    fn get_http_state_mut(&mut self) -> &mut HttpHandlerData;

    /// Get the mutable reference to the events handler data.
    fn get_events_state_mut(&mut self) -> &mut EventHandlerData;

    /// Get the runtime host state for a given resource key.
    fn get_host_state<T: 'static, TTable: 'static>(
        &mut self,
        resource_key: String,
    ) -> (&mut T, &mut TTable);
}
