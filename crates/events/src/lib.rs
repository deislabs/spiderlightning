use std::{
    ops::DerefMut,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::Result;
use crossbeam_utils::thread;

use crate::events::Error;
use crate::events::Observable as GeneratedObservable;
use crossbeam_channel::{unbounded, Receiver, Sender};
use events_api::{AttributesReader, Event, EventHandler, EventParam};

use runtime::{
    impl_resource,
    resource::{
        get_table, Ctx, DataT, Linker, Resource, ResourceMap, ResourceTables, RuntimeResource,
    },
};
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
    host_state: Option<ResourceMap>,
    event_handler: Option<Arc<Mutex<EventHandler<Ctx>>>>,
    store: Option<Arc<Mutex<Store<Ctx>>>>,
}

impl_resource!(
    Events,
    events::EventsTables<Events>,
    ResourceMap,
    SCHEME_NAME.to_string()
);

/// An owned observable
struct Observable {
    rd: String,
    key: String,
    sender: Arc<Mutex<Sender<Event>>>,
    receiver: Arc<Mutex<Receiver<Event>>>,
}

impl From<GeneratedObservable<'_>> for Observable {
    fn from(observable: GeneratedObservable) -> Self {
        let (sender, receiver) = unbounded();
        Self {
            rd: observable.rd.to_string(),
            key: observable.key.to_string(),
            sender: Arc::new(Mutex::new(sender)),
            receiver: Arc::new(Mutex::new(receiver)),
        }
    }
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
        let ob2 = ob.into();
        self.observables.push(ob2);
        Ok(())
    }

    fn events_exec(&mut self, _events: &Self::Events, duration: u64) -> Result<(), Error> {
        for ob in &self.observables {
            // check if observable has changed
            let map = self.host_state.as_mut().unwrap();

            let mut map = map.lock().unwrap();
            let data = map.get::<String>(&ob.rd).unwrap().to_string();
            let resource = &mut map.get_dynamic_mut(&ob.rd).unwrap().0;
            resource.watch(&data, &ob.rd, &ob.key, ob.sender.clone())?;
        }
        thread::scope(|s| -> Result<()> {
            let mut thread_handles = vec![];
            for ob in &self.observables {
                let handler = self.event_handler.as_ref().unwrap().clone();
                let store = self.store.as_mut().unwrap().clone();
                let receiver = ob.receiver.clone();
                let receive_thread = s.spawn(move |_| loop {
                    match receiver
                        .lock()
                        .unwrap()
                        .recv_deadline(Instant::now() + Duration::from_secs(duration))
                    {
                        Ok(mut event) => {
                            let mut store = store.lock().unwrap();
                            let spec = event.specversion();
                            let data: Option<String> = event.take_data().2.map(|d| {
                                d.try_into().unwrap_or_else(|e| {
                                    tracing::error!(
                                        "Failed to convert event data to string: {}",
                                        e
                                    );
                                    "{}".to_string()
                                })
                            });
                            let time = event.time().take().map(|d| d.to_rfc2822());
                            let event_param = EventParam {
                                specversion: spec.as_str(),
                                ty: event.ty(),
                                source: event.source(),
                                id: event.id(),
                                data: data.as_deref().map(|d| d.as_bytes()),
                                datacontenttype: event.datacontenttype(),
                                dataschema: None,
                                subject: event.subject(),
                                time: time.as_deref(),
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
                });
                thread_handles.push(receive_thread);
            }
            for handle in thread_handles {
                handle
                    .join()
                    .expect("internal error: joined thread failed")?;
            }
            Ok(())
        })
        .map_err(|e| {
            anyhow::anyhow!(format!(
                "internal error: joined thread failed with {}",
                e.downcast::<events::Error>().unwrap()
            ))
        })??;
        Ok(())
    }
}
