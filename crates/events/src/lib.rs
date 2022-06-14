use std::{cell::RefCell, rc::Rc};

use anyhow::Result;

use crate::events::{EventError, Observable as GeneratedObservable};
use runtime::resource::{
    event_handler::{EventHandler, EventParam},
    Ctx, Resource, ResourceMap, RuntimeResource,
};
use runtime::resource::{get_table, ResourceTables};
use wasmtime::Store;

use crate::events::add_to_linker;
wit_bindgen_wasmtime::export!("../../wit/events.wit");

const SCHEME_NAME: &str = "events";

/// Events capability
#[derive(Default)]
pub struct Events {
    observables: Vec<Observable>,
    resource_map: Option<ResourceMap>,
    event_handler: Option<EventHandler<Ctx>>,
    store: Option<Rc<RefCell<Store<Ctx>>>>,
}

/// An owned observable
struct Observable {
    rd: String,
    key: String,
}

impl Events {
    /// Host will call this function to update store and event_handler
    pub fn update_state(
        &mut self,
        store: Rc<RefCell<Store<Ctx>>>,
        event_handler: EventHandler<Ctx>,
    ) -> Result<()> {
        self.event_handler = Some(event_handler);
        self.store = Some(store);
        Ok(())
    }
}

impl Resource for Events {
    fn add_resource_map(&mut self, resource_map: ResourceMap) -> Result<()> {
        self.resource_map = Some(resource_map);
        Ok(())
    }

    fn get_inner(&self) -> &dyn std::any::Any {
        unimplemented!("events will not be dynamically dispatched to a specific resource")
    }

    fn changed(&self, _key: &str) -> bool {
        unimplemented!("events will not be listened to")
    }
}

impl RuntimeResource for Events {
    fn add_to_linker(linker: &mut runtime::resource::Linker<runtime::resource::Ctx>) -> Result<()> {
        crate::add_to_linker(linker, |cx| {
            get_table::<Self, events::EventsTables<Self>>(cx, SCHEME_NAME.to_string())
        })
    }

    fn build_data() -> Result<runtime::resource::DataT> {
        let events = Self::default();
        Ok((
            Box::new(events),
            Some(Box::new(events::EventsTables::<Self>::default())),
        ))
    }
}

impl<T> ResourceTables<dyn Resource> for events::EventsTables<T> where T: events::Events + 'static {}

impl events::Events for Events {
    type Events = ();
    fn events_get(&mut self) -> Result<Self::Events, EventError> {
        Ok(())
    }

    fn events_listen(&mut self, _events: &Self::Events, ob: GeneratedObservable<'_>) -> Result<(), EventError> {
        // TODO (Joe): I can't figure out how to not deep copy the Observable here to satisfy the
        // Rust lifetime rules.
        let ob2 = Observable {
            rd: ob.rd.to_string(),
            key: ob.key.to_string(),
        };
        self.observables.push(ob2);
        Ok(())
    }

    fn events_exec(&mut self, _events: &Self::Events, duration: u64) -> Result<(), EventError> {
        // loop until duration time has passed
        let mut duration = duration;
        loop {
            for ob in &self.observables {
                // check if observable has changed
                let map = self
                    .resource_map
                    .as_mut()
                    .ok_or_else(|| anyhow::anyhow!("resource map is not initialized"))?
                    .lock()
                    .unwrap();
                let resource = &map.get_dynamic(&ob.rd)?.0;
                if resource.changed(&ob.key) {
                    // call event handler
                    let event = EventParam {
                        source: &ob.rd,
                        event_type: "changed",
                        specversion: "1",
                        id: "id",
                        data: None,
                    };
                    unsafe {
                        let store = self.store.as_mut().unwrap();
                        let store = &mut (*(*store).as_ptr());
                        match self.event_handler
                            .as_ref()
                            .unwrap()
                            .handle_event(store, event) {
                                Ok(_) => (),
                                Err(e) => {
                                    return Err(events::EventError::Error(format!("event handler error {}", e)));
                                }
                            }
                    }
                }
            }
            // sleep for 1 second
            std::thread::sleep(std::time::Duration::from_secs(1));
            // decrement duration
            duration -= 1;
            // if duration is 0, break
            if duration == 0 {
                break;
            }
        }

        Ok(())
    }
}

impl From<anyhow::Error> for events::EventError {
    fn from(e: anyhow::Error) -> Self {
        Self::Error(e.to_string())
    }
}