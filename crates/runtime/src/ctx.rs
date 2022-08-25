use anyhow::Result;
use slight_common::{Buildable, HostState, ResourceBuilder};
use std::collections::HashMap;

/// A clonable state enum for all the capabilities.
///
/// This is used in the builder to build the runtime context.
#[derive(Clone)]
pub enum State<T: Buildable> {
    Kv(slight_kv::KvState),
    Mq(slight_mq::MqState),
    Http(slight_http::HttpState<T>),
    PubSub(slight_pubsub::PubsubState),
    Lockd(slight_lockd::LockdState),
    RtCfg(slight_runtime_configs::ConfigsState),
    Events(slight_events::EventsState<T>),
}

/// A runtime context for slight capabilities.
///
/// It is a wrapper around a HashMap of HostState, which are
/// generated bindings for the capabilities, and are linked to
/// the `wasmtime::Linker`.
///
/// The `SlightCtx` cannot be created directly, but it can be
/// constructed using the `SlightCtxBuilder`.
///
/// The `SlightCtx` is not cloneable, but the `SlightCtxBuilder` is.
#[derive(Default)]
pub struct SlightCtx(HashMap<String, HostState>);

impl SlightCtx {
    /// Get a reference to the inner HashMap.
    pub fn get_ref(&self) -> &HashMap<String, HostState> {
        &self.0
    }

    /// Get a mutable reference to the inner HashMap.
    pub fn get_mut(&mut self) -> &mut HashMap<String, HostState> {
        &mut self.0
    }
}

/// A builder for the `SlightCtx`.
///
/// It knows how to build the `HostState` given a `State` enum, because it
/// is generic to the `Buildable` trait, which has a `build` method.
#[derive(Clone)]
pub struct SlightCtxBuilder<T: Buildable + Send + Sync> {
    states: Vec<State<T>>,
}

impl<T: Buildable + Send + Sync> Default for SlightCtxBuilder<T> {
    fn default() -> Self {
        Self { states: vec![] }
    }
}

impl<T: Buildable + Send + Sync + 'static> SlightCtxBuilder<T> {
    /// Create a new `SlightCtxBuilder` with empty states.
    pub fn new() -> Self {
        Self { states: Vec::new() }
    }

    /// Add a new state to the builder.
    pub fn add_state(mut self, state: State<T>) -> Result<Self> {
        self.states.push(state);
        Ok(self)
    }

    /// Build the `SlightCtx` from the states.
    pub fn build(self) -> SlightCtx {
        let mut ctx = SlightCtx::default();
        for state in self.states.into_iter() {
            match state {
                State::Kv(state) => {
                    ctx.0
                        .insert("kv".to_string(), slight_kv::Kv::build(state).unwrap());
                }
                State::Http(state) => {
                    ctx.0
                        .insert("http".to_string(), slight_http::Http::build(state).unwrap());
                }
                State::Mq(state) => {
                    ctx.0
                        .insert("mq".to_string(), slight_mq::Mq::build(state).unwrap());
                }
                State::PubSub(state) => {
                    ctx.0.insert(
                        "pubsub".to_string(),
                        slight_pubsub::Pubsub::build(state).unwrap(),
                    );
                }
                State::Lockd(state) => {
                    ctx.0.insert(
                        "lockd".to_string(),
                        slight_lockd::Lockd::build(state).unwrap(),
                    );
                }
                State::RtCfg(state) => {
                    ctx.0.insert(
                        "configs".to_string(),
                        slight_runtime_configs::Configs::build(state).unwrap(),
                    );
                }
                State::Events(state) => {
                    ctx.0.insert(
                        "events".to_string(),
                        slight_events::Events::build(state).unwrap(),
                    );
                }
            };
        }
        ctx
    }
}

#[cfg(test)]
mod unittests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::Builder;
    use anyhow::Result;
    use slight_events_api::StateTable;

    #[test]
    fn test_ctx_builder() -> Result<()> {
        let resource_map = Arc::new(Mutex::new(StateTable::default()));
        let builder = SlightCtxBuilder::<Builder>::new()
            .add_state(State::Kv(slight_kv::KvState::default()))?
            .add_state(State::Mq(slight_mq::MqState::default()))?
            .add_state(State::PubSub(slight_pubsub::PubsubState::default()))?
            .add_state(State::Lockd(slight_lockd::LockdState::default()))?
            .add_state(State::RtCfg(slight_runtime_configs::ConfigsState::default()))?
            .add_state(State::Http(slight_http::HttpState::new(
                resource_map.clone(),
            )))?
            .add_state(State::Events(slight_events::EventsState::new(resource_map)))?;

        let ctx = builder.build();
        assert_eq!(ctx.0.len(), 7);
        assert!(ctx.0.contains_key("kv"));
        assert!(ctx.0.contains_key("mq"));
        assert!(ctx.0.contains_key("pubsub"));
        assert!(ctx.0.contains_key("lockd"));
        assert!(ctx.0.contains_key("configs"));
        assert!(ctx.0.contains_key("http"));
        assert!(ctx.0.contains_key("events"));

        Ok(())
    }
}
