#![feature(deadline_api)]
use std::{
    ops::DerefMut,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use anyhow::Result;
use crossbeam_utils::thread;

use crate::events::Observable as GeneratedObservable;
use events::Error;
use runtime::resource::{
    event_handler::{EventHandler, EventParam},
    Ctx, Event, Resource, ResourceMap, RuntimeResource,
};
use runtime::resource::{get_table, ResourceTables};
use wasmtime::Store;

use crate::events::add_to_linker;
wit_bindgen_wasmtime::export!("../../wit/events.wit");
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);

const SCHEME_NAME: &str = "events";

/// Events capability
#[derive(Default)]
pub struct Events {
    observables: Vec<Observable>,
    resource_map: Option<ResourceMap>,
    event_handler: Option<Arc<Mutex<EventHandler<Ctx>>>>,
    store: Option<Arc<Mutex<Store<Ctx>>>>,
}

/// An owned observable
struct Observable {
    rd: String,
    key: String,
    sender: Arc<Mutex<Sender<Event>>>,
    receiver: Arc<Mutex<Receiver<Event>>>,
}

impl Events {
    /// Host will call this function to update store and event_handler
    pub fn update_state(
        &mut self,
        store: Arc<Mutex<Store<Ctx>>>,
        event_handler: Arc<Mutex<EventHandler<Ctx>>>,
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

    fn watch(
        &mut self,
        _data: &str,
        _rd: &str,
        _key: &str,
        _sender: Arc<Mutex<Sender<Event>>>,
    ) -> Result<()> {
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
    fn events_get(&mut self) -> Result<Self::Events, Error> {
        Ok(())
    }

    fn events_listen(
        &mut self,
        _events: &Self::Events,
        ob: GeneratedObservable<'_>,
    ) -> Result<(), Error> {
        // TODO (Joe): I can't figure out how to not deep copy the Observable here to satisfy the
        // Rust lifetime rules.
        let (sender, receiver) = channel();
        let ob2 = Observable {
            rd: ob.rd.to_string(),
            key: ob.key.to_string(),
            sender: Arc::new(Mutex::new(sender)),
            receiver: Arc::new(Mutex::new(receiver)),
        };
        self.observables.push(ob2);
        Ok(())
    }

    fn events_exec(&mut self, _events: &Self::Events, duration: u64) -> Result<(), Error> {
        thread::scope(|s| {
            let mut thread_handles = vec![];
            // loop until duration time has passed
            let duration = duration;
            for ob in &self.observables {
                // check if observable has changed
                let map = self.resource_map.as_mut().unwrap().clone();
                let rd = ob.rd.clone();
                let key = ob.key.clone();
                let sender = ob.sender.clone();
                thread_handles.push(s.spawn(move |_| {
                    let mut map = map.lock().unwrap();
                    let data = map.get::<String>(&ob.rd).unwrap().to_string();
                    let resource = &mut map.get_dynamic_mut(&rd).unwrap().0;
                    resource.watch(&data, &rd, &key, sender)?;
                    Ok(())
                }));
            }

            for ob in &self.observables {
                let handler = self.event_handler.as_ref().unwrap().clone();
                let store = self.store.as_mut().unwrap().clone();
                let receiver = ob.receiver.clone();
                thread_handles.push(s.spawn(move |_| loop {
                    match receiver
                        .lock()
                        .unwrap()
                        .recv_deadline(Instant::now() + Duration::from_secs(duration))
                    {
                        Ok(event) => {
                            let mut store = store.lock().unwrap();
                            let event_param = EventParam {
                                specversion: event.specversion.as_str(),
                                event_type: event.event_type.as_str(),
                                source: event.source.as_str(),
                                id: event.id.as_str(),
                                data: event.data.as_deref(),
                            };
                            match handler
                                .lock()
                                .unwrap()
                                .handle_event(store.deref_mut(), event_param)
                            {
                                Ok(_) => (),
                                Err(e) => {
                                    return Err(events::Error::ErrorWithDescription(format!(
                                        "event handler error {}",
                                        e
                                    )));
                                }
                            }
                        }
                        Err(_) => return Ok(()),
                    }
                }));
            }
            for handle in thread_handles {
                handle.join().unwrap();
            }
        })
        .unwrap();
        Ok(())
    }
}
