use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use async_trait::async_trait;
use crossbeam_utils::thread;
use events::EventsTables;

use crate::events::Error;
use crate::events::Observable as GeneratedObservable;
use crossbeam_channel::{unbounded, Receiver, Sender};
use futures::executor::block_on;
use slight_common::{impl_resource, Buildable, Builder, Ctx};
use slight_events_api::{AttributesReader, Event, EventHandler, EventParam, ResourceMap};
use uuid::Uuid;

wit_bindgen_wasmtime::export!({paths: ["../../wit/events.wit"], async: *});
wit_error_rs::impl_error!(Error);
wit_error_rs::impl_from!(anyhow::Error, Error::ErrorWithDescription);

/// Events capability
#[derive(Default, Clone)]
pub struct Events<T: Buildable> {
    resource_map: ResourceMap,
    builder: Option<Builder<T>>,
}

impl<T: Buildable> Events<T> {
    pub fn new(resource_map: ResourceMap) -> Self {
        Self {
            resource_map,
            builder: None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct EventsGuest {
    observables: Vec<Observable>,
}

/// An owned observable
#[derive(Clone, Debug)]
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

impl<T: Buildable> Events<T> {
    /// Host will call this function to update store and event_handler
    pub fn update_state(&mut self, builder: Builder<T>) -> Result<()> {
        self.builder = Some(builder);
        Ok(())
    }
}

#[async_trait]
impl<T: Buildable + Send + Sync + 'static> events::Events for Events<T> {
    type Events = EventsGuest;
    async fn events_get(&mut self) -> Result<Self::Events, Error> {
        Ok(Default::default())
    }

    async fn events_listen(
        &mut self,
        self_: &Self::Events,
        ob: GeneratedObservable<'_>,
    ) -> Result<Self::Events, Error> {
        Uuid::parse_str(ob.rd)
            .with_context(|| "internal error: failed to parse internal handle to this resource")?;
        let ob = ob.into();
        // FIXME: the reason I had to clone the observable is because the observable is owned by
        // self_ which is not a mutable reference.
        let mut observables = self_.observables.clone();
        observables.push(ob);
        Ok(Self::Events { observables })
    }

    async fn events_exec(&mut self, self_: &Self::Events, duration: u64) -> Result<(), Error> {
        for ob in &self_.observables {
            // check if observable has changed

            let map = self.resource_map.clone();

            let mut map = map.lock().unwrap();
            let resource = map.get_mut(&ob.rd).unwrap();
            resource.watch(&ob.key, ob.sender.clone())?;
        }
        thread::scope(|s| -> Result<()> {
            let mut thread_handles = vec![];
            for ob in &self_.observables {
                let builder = self.builder.as_ref().unwrap().clone();
                let receiver = ob.receiver.clone();
                let receive_thread = s.spawn(move |_| loop {
                    let recv = receiver
                        .lock()
                        .unwrap()
                        .recv_deadline(Instant::now() + Duration::from_secs(duration));
                    match recv {
                        Ok(mut event) => {
                            let (mut store, instance) = block_on(builder.inner().build());
                            let handler = EventHandler::new(&mut store, &instance, |ctx| {
                                ctx.get_events_state_mut()
                            })?;
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
                            let event_res = block_on(handler.handle_event(&mut store, event_param));
                            match event_res {
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

impl_resource!(
    Events<T>,
    EventsTables<Events<T>>,
    EventsState<T>,
    T,
    events::add_to_linker,
    "events".to_string()
);
