pub mod implementors;

/// The `SCHEME_NAME` defines the name under which a resource is
/// identifiable by in a `ResourceMap`.
const SCHEME_NAME: &str = "configs";

use std::sync::{Arc, Mutex};

use anyhow::Result;
use crossbeam_channel::Sender;
use events_api::Event;
use uuid::Uuid;

use implementors::{envvars::EnvVars, usersecrets::UserSecrets};
use runtime::{impl_resource, resource::BasicState};

/// It is mandatory to `use <interface>::*` due to `impl_resource!`.
/// That is because `impl_resource!` accesses the `crate`'s
/// `add_to_linker`, and not the `<interface>::add_to_linker` directly.
use configs::*;
wit_bindgen_wasmtime::export!("../../wit/configs.wit");
wit_error_rs::impl_error!(configs::Error);
wit_error_rs::impl_from!(anyhow::Error, configs::Error::ErrorWithDescription);

/// The `Configs` structure is what will implement the `configs::Configs` trait
/// coming from the generated code of off `configs.wit`.
///
/// It maintains a `host_state`.
pub struct Configs {
    host_state: ConfigsState,
}

impl_resource!(
    Configs,
    configs::ConfigsTables<Configs>,
    ConfigsState,
    SCHEME_NAME.to_string()
);

/// This is the type of the `host_state` property from our `Configs` structure.
///
/// It holds:
///     - a `lockd_implementor` `String` — this comes directly from a
///     user's `slightfile` and it is what allows us to dynamically
///     dispatch to a specific implementor's implentation, and
///     - the `slight_state` (of type `BasicState`) that contains common
///     things received from the slight binary (i.e., the `resource_map`,
///     the `config_type`, and the `config_toml_file_path`).
pub struct ConfigsState {
    pub configs_implementor: String,
    pub slight_state: BasicState,
}

impl ConfigsState {
    pub fn new(configs_implementor: String, slight_state: BasicState) -> Self {
        Self {
            configs_implementor,
            slight_state,
        }
    }
}

impl configs::Configs for Configs {
    type Configs = ConfigsInner;

    fn configs_open(&mut self) -> Result<Self::Configs, configs::Error> {
        // populate our inner configs object w/ the state received from `slight`
        // (i.e., what type of configs implementor we are using), and the assigned
        // name of the object.
        let inner = Self::Configs::new(
            &self.host_state.configs_implementor,
            &self.host_state.slight_state,
        );

        self.host_state
            .slight_state
            .resource_map
            .lock()
            .unwrap()
            .set(inner.resource_descriptor.clone(), Box::new(inner.clone()));

        Ok(inner)
    }

    fn configs_get(&mut self, self_: &Self::Configs, key: &str) -> Result<Vec<u8>, configs::Error> {
        Ok(match &self_.configs_implementor {
            ConfigsImplementor::EnvVars => EnvVars::get(key)?,
            ConfigsImplementor::UserSecrets => {
                UserSecrets::get(key, &self.host_state.slight_state.config_toml_file_path)?
            }
        })
    }

    fn configs_set(
        &mut self,
        self_: &Self::Configs,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), configs::Error> {
        match &self_.configs_implementor {
            ConfigsImplementor::EnvVars => EnvVars::set(key, value)?,
            ConfigsImplementor::UserSecrets => UserSecrets::set(
                key,
                value,
                &self.host_state.slight_state.config_toml_file_path,
            )?,
        };

        Ok(())
    }
}

/// This is the type of the associated type coming from the `configs::Configs` trait
/// implementation.
///
/// It holds:
///     - a `configs_implementor` (i.e., a variant `ConfigsImplementor` `enum`), and
///     - a `resource_descriptor` (i.e., an UUID that uniquely identifies
///     resource's instance).
///
/// It must `derive`:
///     - `Debug` due to a constraint on the associated type.
///     - `Clone` because the `ResourceMap` it will be added onto,
///     must own its' data.
///
/// It must be public because the implementation of `configs::Configs` cannot leak
/// a private type.
#[derive(Debug, Clone)]
pub struct ConfigsInner {
    configs_implementor: ConfigsImplementor,
    resource_descriptor: String,
}

impl ConfigsInner {
    fn new(configs_implementor: &str, slight_state: &BasicState) -> Self {
        Self {
            configs_implementor: ConfigsImplementor::new(configs_implementor, slight_state),
            resource_descriptor: Uuid::new_v4().to_string(),
        }
    }
}

impl runtime::resource::Watch for ConfigsInner {
    fn watch(&mut self, key: &str, sender: Arc<Mutex<Sender<Event>>>) -> Result<()> {
        todo!(
            "got {} and {:?}, but got nothing to do with it yet",
            key,
            sender
        );
    }
}

/// This defines the available implementor implementations for the `Configs` interface.
///
/// As per its' usage in `ConfigsInner`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone)]
pub enum ConfigsImplementor {
    EnvVars,
    UserSecrets, // user creates configs at compile time that are encrypted and stored in their slightfile
}

impl From<ConfigsImplementor> for String {
    fn from(from_ct: ConfigsImplementor) -> Self {
        match from_ct {
            ConfigsImplementor::UserSecrets => "configs.usersecrets".to_string(),
            ConfigsImplementor::EnvVars => "configs.envvars".to_string(),
        }
    }
}

impl From<&str> for ConfigsImplementor {
    fn from(from_str: &str) -> Self {
        match from_str {
            "configs.usersecrets" => ConfigsImplementor::UserSecrets,
            "configs.envvars" => ConfigsImplementor::EnvVars,
            _ => panic!("Unknown config type: {}", from_str),
        }
    }
}

impl Default for ConfigsImplementor {
    fn default() -> Self {
        ConfigsImplementor::EnvVars
    }
}

impl ConfigsImplementor {
    fn new(configs_implementor: &str, _: &BasicState) -> Self {
        match configs_implementor {
            "configs.envvars" => Self::EnvVars,
            "configs.usersecrets" => Self::UserSecrets,
            p => panic!(
                "failed to match provided configs name (i.e., '{}' to any known host implementations",
                p
            ),
        }
    }
}

/// SDK-ish bit
pub fn get(config_type: &str, key: &str, toml_file_path: &str) -> Result<Vec<u8>> {
    match config_type.into() {
        ConfigsImplementor::EnvVars => Ok(EnvVars::get(key)?),
        ConfigsImplementor::UserSecrets => Ok(UserSecrets::get(key, toml_file_path)?),
    }
}

pub fn set(config_type: &str, key: &str, value: &[u8], toml_file_path: &str) -> Result<()> {
    match config_type.into() {
        ConfigsImplementor::EnvVars => Ok(EnvVars::set(key, value)?),
        ConfigsImplementor::UserSecrets => Ok(UserSecrets::set(key, value, toml_file_path)?),
    }
}
