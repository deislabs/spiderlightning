#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use anyhow::{Context, Result};
use crossbeam_utils::thread;
use events::EventsTables;
use crate::events::Error;
use crate::events::Observable as GeneratedObservable;
use crossbeam_channel::{unbounded, Receiver, Sender};
use slight_common::{Buildable, Builder, Ctx, impl_resource};
use slight_events_api::{AttributesReader, Event, EventHandler, EventParam, ResourceMap};
use uuid::Uuid;
pub mod events {
    #[allow(unused_imports)]
    use wit_bindgen_wasmtime::{wasmtime, anyhow};
    pub struct Observable<'a> {
        pub rd: &'a str,
        pub key: &'a str,
    }
    #[automatically_derived]
    impl<'a> ::core::clone::Clone for Observable<'a> {
        #[inline]
        fn clone(&self) -> Observable<'a> {
            Observable {
                rd: ::core::clone::Clone::clone(&self.rd),
                key: ::core::clone::Clone::clone(&self.key),
            }
        }
    }
    impl<'a> std::fmt::Debug for Observable<'a> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Observable")
                .field("rd", &self.rd)
                .field("key", &self.key)
                .finish()
        }
    }
    pub enum Error {
        ErrorWithDescription(String),
    }
    #[automatically_derived]
    impl ::core::clone::Clone for Error {
        #[inline]
        fn clone(&self) -> Error {
            match self {
                Error::ErrorWithDescription(__self_0) => {
                    Error::ErrorWithDescription(::core::clone::Clone::clone(__self_0))
                }
            }
        }
    }
    impl std::fmt::Debug for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Error::ErrorWithDescription(e) => f
                    .debug_tuple("Error::ErrorWithDescription")
                    .field(e)
                    .finish(),
            }
        }
    }
    pub trait Events: Sized {
        type Events: std::fmt::Debug;
        fn events_get(&mut self) -> Result<Self::Events, Error>;
        fn events_listen(
            &mut self,
            self_: &Self::Events,
            ob: Observable<'_>,
        ) -> Result<Self::Events, Error>;
        fn events_exec(&mut self, self_: &Self::Events, duration: u64) -> Result<(), Error>;
        fn drop_events(&mut self, state: Self::Events) {
            drop(state);
        }
    }
    pub struct EventsTables<T: Events> {
        pub(crate) events_table: wit_bindgen_wasmtime::Table<T::Events>,
    }
    impl<T: Events> Default for EventsTables<T> {
        fn default() -> Self {
            Self {
                events_table: Default::default(),
            }
        }
    }
    pub fn add_to_linker<T, U>(
        linker: &mut wasmtime::Linker<T>,
        get: impl Fn(&mut T) -> (&mut U, &mut EventsTables<U>) + Send + Sync + Copy + 'static,
    ) -> anyhow::Result<()>
    where
        U: Events,
    {
        use wit_bindgen_wasmtime::rt::get_memory;
        use wit_bindgen_wasmtime::rt::get_func;
        linker.func_wrap(
            "events",
            "events::get",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let host = get(caller.data_mut());
                let (host, _tables) = host;
                let result = host.events_get();
                match result {
                    Ok(e) => {
                        let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                        let (_, _tables) = get(data);
                        caller_memory
                            .store(arg0 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                        caller_memory.store(
                            arg0 + 4,
                            wit_bindgen_wasmtime::rt::as_i32(_tables.events_table.insert(e) as i32),
                        )?;
                    }
                    Err(e) => {
                        let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                        let (_, _tables) = get(data);
                        caller_memory
                            .store(arg0 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                        match e {
                            Error::ErrorWithDescription(e) => {
                                caller_memory.store(
                                    arg0 + 4,
                                    wit_bindgen_wasmtime::rt::as_i32(0i32) as u8,
                                )?;
                                let vec0 = e;
                                let ptr0 = func_canonical_abi_realloc
                                    .call(&mut caller, (0, 0, 1, vec0.len() as i32))?;
                                let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                                let (_, _tables) = get(data);
                                caller_memory.store_many(ptr0, vec0.as_bytes())?;
                                caller_memory.store(
                                    arg0 + 12,
                                    wit_bindgen_wasmtime::rt::as_i32(vec0.len() as i32),
                                )?;
                                caller_memory
                                    .store(arg0 + 8, wit_bindgen_wasmtime::rt::as_i32(ptr0))?;
                            }
                        };
                    }
                };
                Ok(())
            },
        )?;
        linker.func_wrap(
            "events",
            "events::listen",
            move |mut caller: wasmtime::Caller<'_, T>,
                  arg0: i32,
                  arg1: i32,
                  arg2: i32,
                  arg3: i32,
                  arg4: i32,
                  arg5: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let (mem, data) = memory.data_and_store_mut(&mut caller);
                let mut _bc = wit_bindgen_wasmtime::BorrowChecker::new(mem);
                let host = get(data);
                let (host, _tables) = host;
                let ptr0 = arg1;
                let len0 = arg2;
                let ptr1 = arg3;
                let len1 = arg4;
                let param0 = _tables
                    .events_table
                    .get((arg0) as u32)
                    .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                let param1 = Observable {
                    rd: _bc.slice_str(ptr0, len0)?,
                    key: _bc.slice_str(ptr1, len1)?,
                };
                let result = host.events_listen(param0, param1);
                match result {
                    Ok(e) => {
                        let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                        let (_, _tables) = get(data);
                        caller_memory
                            .store(arg5 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                        caller_memory.store(
                            arg5 + 4,
                            wit_bindgen_wasmtime::rt::as_i32(_tables.events_table.insert(e) as i32),
                        )?;
                    }
                    Err(e) => {
                        let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                        let (_, _tables) = get(data);
                        caller_memory
                            .store(arg5 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                        match e {
                            Error::ErrorWithDescription(e) => {
                                caller_memory.store(
                                    arg5 + 4,
                                    wit_bindgen_wasmtime::rt::as_i32(0i32) as u8,
                                )?;
                                let vec2 = e;
                                let ptr2 = func_canonical_abi_realloc
                                    .call(&mut caller, (0, 0, 1, vec2.len() as i32))?;
                                let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                                let (_, _tables) = get(data);
                                caller_memory.store_many(ptr2, vec2.as_bytes())?;
                                caller_memory.store(
                                    arg5 + 12,
                                    wit_bindgen_wasmtime::rt::as_i32(vec2.len() as i32),
                                )?;
                                caller_memory
                                    .store(arg5 + 8, wit_bindgen_wasmtime::rt::as_i32(ptr2))?;
                            }
                        };
                    }
                };
                Ok(())
            },
        )?;
        linker.func_wrap(
            "events",
            "events::exec",
            move |mut caller: wasmtime::Caller<'_, T>, arg0: i32, arg1: i64, arg2: i32| {
                let func = get_func(&mut caller, "canonical_abi_realloc")?;
                let func_canonical_abi_realloc =
                    func.typed::<(i32, i32, i32, i32), i32, _>(&caller)?;
                let memory = &get_memory(&mut caller, "memory")?;
                let host = get(caller.data_mut());
                let (host, _tables) = host;
                let param0 = _tables
                    .events_table
                    .get((arg0) as u32)
                    .ok_or_else(|| wasmtime::Trap::new("invalid handle index"))?;
                let param1 = arg1 as u64;
                let result = host.events_exec(param0, param1);
                match result {
                    Ok(e) => {
                        let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                        let (_, _tables) = get(data);
                        caller_memory
                            .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(0i32) as u8)?;
                        let () = e;
                    }
                    Err(e) => {
                        let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                        let (_, _tables) = get(data);
                        caller_memory
                            .store(arg2 + 0, wit_bindgen_wasmtime::rt::as_i32(1i32) as u8)?;
                        match e {
                            Error::ErrorWithDescription(e) => {
                                caller_memory.store(
                                    arg2 + 4,
                                    wit_bindgen_wasmtime::rt::as_i32(0i32) as u8,
                                )?;
                                let vec0 = e;
                                let ptr0 = func_canonical_abi_realloc
                                    .call(&mut caller, (0, 0, 1, vec0.len() as i32))?;
                                let (caller_memory, data) = memory.data_and_store_mut(&mut caller);
                                let (_, _tables) = get(data);
                                caller_memory.store_many(ptr0, vec0.as_bytes())?;
                                caller_memory.store(
                                    arg2 + 12,
                                    wit_bindgen_wasmtime::rt::as_i32(vec0.len() as i32),
                                )?;
                                caller_memory
                                    .store(arg2 + 8, wit_bindgen_wasmtime::rt::as_i32(ptr0))?;
                            }
                        };
                    }
                };
                Ok(())
            },
        )?;
        linker.func_wrap(
            "canonical_abi",
            "resource_drop_events",
            move |mut caller: wasmtime::Caller<'_, T>, handle: u32| {
                let (host, tables) = get(caller.data_mut());
                let handle = tables.events_table.remove(handle).map_err(|e| {
                    wasmtime::Trap::new({
                        let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                            &["failed to remove handle: "],
                            &[::core::fmt::ArgumentV1::new_display(&e)],
                        ));
                        res
                    })
                })?;
                host.drop_events(handle);
                Ok(())
            },
        )?;
        Ok(())
    }
    use wit_bindgen_wasmtime::rt::RawMem;
}
const _ : & str = "use { observable } from resources\nuse { error } from types\n\n\nresource events {\n\tstatic get: function() -> expected<events, error>\n\tlisten: function(ob: observable) -> expected<events, error>\n\texec: function(duration: u64) -> expected<unit, error>\n}\n" ;
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_fmt(::core::fmt::Arguments::new_v1(
            &[""],
            &[::core::fmt::ArgumentV1::new_debug(&&self)],
        ))
    }
}
impl std::error::Error for Error {}
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::ErrorWithDescription(err.to_string())
    }
}
/// Events capability
pub struct Events<T: Buildable> {
    host_state: EventsState<T>,
}
#[automatically_derived]
impl<T: ::core::default::Default + Buildable> ::core::default::Default for Events<T> {
    #[inline]
    fn default() -> Events<T> {
        Events {
            host_state: ::core::default::Default::default(),
        }
    }
}
impl<T: Buildable> Events<T> {
    pub fn from_state(host_state: EventsState<T>) -> Self {
        Self { host_state }
    }
}
pub struct EventsState<T: Buildable> {
    resource_map: ResourceMap,
    builder: Option<Builder<T>>,
}
#[automatically_derived]
impl<T: ::core::default::Default + Buildable> ::core::default::Default for EventsState<T> {
    #[inline]
    fn default() -> EventsState<T> {
        EventsState {
            resource_map: ::core::default::Default::default(),
            builder: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
impl<T: ::core::clone::Clone + Buildable> ::core::clone::Clone for EventsState<T> {
    #[inline]
    fn clone(&self) -> EventsState<T> {
        EventsState {
            resource_map: ::core::clone::Clone::clone(&self.resource_map),
            builder: ::core::clone::Clone::clone(&self.builder),
        }
    }
}
impl<T: Buildable> EventsState<T> {
    pub fn new(resource_map: ResourceMap) -> Self {
        Self {
            resource_map,
            builder: None,
        }
    }
}
pub struct EventsGuest {
    observables: Vec<Observable>,
}
#[automatically_derived]
impl ::core::clone::Clone for EventsGuest {
    #[inline]
    fn clone(&self) -> EventsGuest {
        EventsGuest {
            observables: ::core::clone::Clone::clone(&self.observables),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for EventsGuest {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field1_finish(
            f,
            "EventsGuest",
            "observables",
            &&self.observables,
        )
    }
}
#[automatically_derived]
impl ::core::default::Default for EventsGuest {
    #[inline]
    fn default() -> EventsGuest {
        EventsGuest {
            observables: ::core::default::Default::default(),
        }
    }
}
/// An owned observable
struct Observable {
    rd: String,
    key: String,
    sender: Arc<Mutex<Sender<Event>>>,
    receiver: Arc<Mutex<Receiver<Event>>>,
}
#[automatically_derived]
impl ::core::clone::Clone for Observable {
    #[inline]
    fn clone(&self) -> Observable {
        Observable {
            rd: ::core::clone::Clone::clone(&self.rd),
            key: ::core::clone::Clone::clone(&self.key),
            sender: ::core::clone::Clone::clone(&self.sender),
            receiver: ::core::clone::Clone::clone(&self.receiver),
        }
    }
}
#[automatically_derived]
impl ::core::fmt::Debug for Observable {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::debug_struct_field4_finish(
            f,
            "Observable",
            "rd",
            &&self.rd,
            "key",
            &&self.key,
            "sender",
            &&self.sender,
            "receiver",
            &&self.receiver,
        )
    }
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
        self.host_state.builder = Some(builder);
        Ok(())
    }
}
impl<T: Buildable + Send + Sync + 'static> events::Events for Events<T> {
    type Events = EventsGuest;
    fn events_get(&mut self) -> Result<Self::Events, Error> {
        Ok(Default::default())
    }
    fn events_listen(
        &mut self,
        self_: &Self::Events,
        ob: GeneratedObservable<'_>,
    ) -> Result<Self::Events, Error> {
        Uuid::parse_str(ob.rd)
            .with_context(|| "internal error: failed to parse internal handle to this resource")?;
        let ob = ob.into();
        let mut observables = self_.observables.clone();
        observables.push(ob);
        Ok(Self::Events { observables })
    }
    fn events_exec(&mut self, self_: &Self::Events, duration: u64) -> Result<(), Error> {
        for ob in &self_.observables {
            let map = self.host_state.resource_map.clone();
            let mut map = map.lock().unwrap();
            let resource = map.get_mut(&ob.rd).unwrap();
            resource.watch(&ob.key, ob.sender.clone())?;
        }
        thread :: scope (| s | -> Result < () > { let mut thread_handles = :: alloc :: vec :: Vec :: new () ; for ob in & self_ . observables { let builder = self . host_state . builder . as_ref () . unwrap () . clone () ; let receiver = ob . receiver . clone () ; let receive_thread = s . spawn (move | _ | loop { let recv = receiver . lock () . unwrap () . recv_deadline (Instant :: now () + Duration :: from_secs (duration)) ; match recv { Ok (mut event) => { let (mut store , instance) = builder . inner () . build () ; let handler = EventHandler :: new (& mut store , & instance , | ctx | { ctx . get_events_state_mut () }) ? ; let spec = event . specversion () ; let data : Option < String > = event . take_data () . 2 . map (| d | { d . try_into () . unwrap_or_else (| e | { { use :: tracing :: __macro_support :: Callsite as _ ; static CALLSITE : :: tracing :: callsite :: DefaultCallsite = { static META : :: tracing :: Metadata < 'static > = { :: tracing_core :: metadata :: Metadata :: new ("event crates/events/src/lib.rs:132" , "slight_events" , :: tracing :: Level :: ERROR , Some ("crates/events/src/lib.rs") , Some (132u32) , Some ("slight_events") , :: tracing_core :: field :: FieldSet :: new (& ["message"] , :: tracing_core :: callsite :: Identifier (& CALLSITE)) , :: tracing :: metadata :: Kind :: EVENT) } ; :: tracing :: callsite :: DefaultCallsite :: new (& META) } ; let enabled = :: tracing :: Level :: ERROR <= :: tracing :: level_filters :: STATIC_MAX_LEVEL && :: tracing :: Level :: ERROR <= :: tracing :: level_filters :: LevelFilter :: current () && { let interest = CALLSITE . interest () ; ! interest . is_never () && :: tracing :: __macro_support :: __is_enabled (CALLSITE . metadata () , interest) } ; if enabled { (| value_set : :: tracing :: field :: ValueSet | { let meta = CALLSITE . metadata () ; :: tracing :: Event :: dispatch (meta , & value_set) ; if match :: tracing :: Level :: ERROR { :: tracing :: Level :: ERROR => :: tracing :: log :: Level :: Error , :: tracing :: Level :: WARN => :: tracing :: log :: Level :: Warn , :: tracing :: Level :: INFO => :: tracing :: log :: Level :: Info , :: tracing :: Level :: DEBUG => :: tracing :: log :: Level :: Debug , _ => :: tracing :: log :: Level :: Trace , } <= :: tracing :: log :: STATIC_MAX_LEVEL { if ! :: tracing :: dispatcher :: has_been_set () { { use :: tracing :: log ; let level = match :: tracing :: Level :: ERROR { :: tracing :: Level :: ERROR => :: tracing :: log :: Level :: Error , :: tracing :: Level :: WARN => :: tracing :: log :: Level :: Warn , :: tracing :: Level :: INFO => :: tracing :: log :: Level :: Info , :: tracing :: Level :: DEBUG => :: tracing :: log :: Level :: Debug , _ => :: tracing :: log :: Level :: Trace , } ; if level <= log :: max_level () { let meta = CALLSITE . metadata () ; let log_meta = log :: Metadata :: builder () . level (level) . target (meta . target ()) . build () ; let logger = log :: logger () ; if logger . enabled (& log_meta) { :: tracing :: __macro_support :: __tracing_log (meta , logger , log_meta , & value_set) } } } } else { { } } } else { { } } ; }) ({ # [allow (unused_imports)] use :: tracing :: field :: { debug , display , Value } ; let mut iter = CALLSITE . metadata () . fields () . iter () ; CALLSITE . metadata () . fields () . value_set (& [(& iter . next () . expect ("FieldSet corrupted (this is a bug)") , Some (& :: core :: fmt :: Arguments :: new_v1 (& ["Failed to convert event data to string: "] , & [:: core :: fmt :: ArgumentV1 :: new_display (& e)]) as & Value))]) }) ; } else { if match :: tracing :: Level :: ERROR { :: tracing :: Level :: ERROR => :: tracing :: log :: Level :: Error , :: tracing :: Level :: WARN => :: tracing :: log :: Level :: Warn , :: tracing :: Level :: INFO => :: tracing :: log :: Level :: Info , :: tracing :: Level :: DEBUG => :: tracing :: log :: Level :: Debug , _ => :: tracing :: log :: Level :: Trace , } <= :: tracing :: log :: STATIC_MAX_LEVEL { if ! :: tracing :: dispatcher :: has_been_set () { { use :: tracing :: log ; let level = match :: tracing :: Level :: ERROR { :: tracing :: Level :: ERROR => :: tracing :: log :: Level :: Error , :: tracing :: Level :: WARN => :: tracing :: log :: Level :: Warn , :: tracing :: Level :: INFO => :: tracing :: log :: Level :: Info , :: tracing :: Level :: DEBUG => :: tracing :: log :: Level :: Debug , _ => :: tracing :: log :: Level :: Trace , } ; if level <= log :: max_level () { let meta = CALLSITE . metadata () ; let log_meta = log :: Metadata :: builder () . level (level) . target (meta . target ()) . build () ; let logger = log :: logger () ; if logger . enabled (& log_meta) { :: tracing :: __macro_support :: __tracing_log (meta , logger , log_meta , & { # [allow (unused_imports)] use :: tracing :: field :: { debug , display , Value } ; let mut iter = CALLSITE . metadata () . fields () . iter () ; CALLSITE . metadata () . fields () . value_set (& [(& iter . next () . expect ("FieldSet corrupted (this is a bug)") , Some (& :: core :: fmt :: Arguments :: new_v1 (& ["Failed to convert event data to string: "] , & [:: core :: fmt :: ArgumentV1 :: new_display (& e)]) as & Value))]) }) } } } } else { { } } } else { { } } ; } } ; "{}" . to_string () }) }) ; let time = event . time () . take () . map (| d | d . to_rfc2822 ()) ; let event_param = EventParam { specversion : spec . as_str () , ty : event . ty () , source : event . source () , id : event . id () , data : data . as_deref () . map (| d | d . as_bytes ()) , datacontenttype : event . datacontenttype () , dataschema : None , subject : event . subject () , time : time . as_deref () , } ; let event_res = handler . handle_event (& mut store , event_param) ; match event_res { Ok (_) => () , Err (e) => { return Err (events :: Error :: ErrorWithDescription ({ let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["event handler error "] , & [:: core :: fmt :: ArgumentV1 :: new_display (& e)])) ; res })) ; } } } Err (_) => return Ok (()) , } }) ; thread_handles . push (receive_thread) ; } for handle in thread_handles { handle . join () . expect ("internal error: joined thread failed") ? ; } Ok (()) }) . map_err (| e | { :: anyhow :: private :: must_use ({ use :: anyhow :: private :: kind :: * ; let error = match { let res = :: alloc :: fmt :: format (:: core :: fmt :: Arguments :: new_v1 (& ["internal error: joined thread failed with "] , & [:: core :: fmt :: ArgumentV1 :: new_display (& e . downcast :: < events :: Error > () . unwrap ())])) ; res } { error => (& error) . anyhow_kind () . new (error) , } ; error }) }) ? ? ;
        Ok(())
    }
}
impl<T: slight_common::Buildable + 'static> slight_common::Resource for Events<T> {}
impl<T: slight_common::Buildable + Send + Sync + 'static>
    slight_common::ResourceTables<dyn slight_common::Resource> for EventsTables<Events<T>>
{
}
impl<T: slight_common::Buildable + Send + Sync + 'static> slight_common::ResourceBuilder
    for Events<T>
{
    type State = EventsState<T>;
    fn build(state: Self::State) -> anyhow::Result<slight_common::HostState> {
        let mut resource = Self { host_state: state };
        Ok((
            Box::new(resource),
            Some(Box::new(<EventsTables<Events<T>>>::default())),
        ))
    }
}
