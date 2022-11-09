use slight_events_api::EventHandlerData;
use slight_http_api::HttpHandlerData;

pub trait Ctx {
    fn get_http_state_mut(&mut self) -> &mut HttpHandlerData;
    fn get_events_state_mut(&mut self) -> &mut EventHandlerData;

    /// Get the runtime host state for a given resource key.
    fn get_host_state<T: 'static, TTable: 'static>(
        &mut self,
        resource_key: String,
    ) -> (&mut T, &mut TTable);
}
