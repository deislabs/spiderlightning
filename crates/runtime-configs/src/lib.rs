use anyhow::Result;
use configs::*;
use crossbeam_channel::Sender;
use events_api::Event;
use proc_macro_utils::{Resource, Watch};
use runtime::{
    impl_resource,
    resource::{
        get_table, Ctx, HostState, Linker, Resource, ResourceBuilder, ResourceMap, ResourceTables,
        Watch,
    },
};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
wit_bindgen_wasmtime::export!("../../wit/configs.wit");
wit_error_rs::impl_error!(configs::Error);
wit_error_rs::impl_from!(anyhow::Error, configs::Error::ErrorWithDescription);

pub mod providers;

const SCHEME_NAME: &str = "configs";

// Struct Representer for wit_bindgen's Config
#[derive(Default, Clone, Resource)]
pub struct Configs {
    host_state: ConfigsState,
}

#[derive(Clone, Watch, Debug)]
pub struct ConfigsInner {
    config_type: Arc<ConfigType>,
}

#[derive(Clone, Default)]
pub struct ConfigsState {
    pub resource_map: ResourceMap,
    pub config_type: String,
    pub config_toml_file_path: String,
}

impl ConfigsState {
    pub fn new(resource_map: ResourceMap, config_type: &str, config_toml_file_path: &str) -> Self {
        Self {
            resource_map,
            config_type: config_type.to_string(),
            config_toml_file_path: config_toml_file_path.to_string(),
        }
    }
}

impl_resource!(
    Configs,
    configs::ConfigsTables<Configs>,
    ConfigsState,
    SCHEME_NAME.to_string()
);

// Currently supported configuration types
#[derive(Clone, Debug, Copy)]
pub enum ConfigType {
    EnvVars,
    UserSecrets, // user creates configs at compile time that are encrypted and stored in the toml file
}

impl From<ConfigType> for String {
    fn from(from_ct: ConfigType) -> Self {
        match from_ct {
            ConfigType::UserSecrets => "usersecrets_configs".to_string(),
            ConfigType::EnvVars => "envvars_configs".to_string(),
        }
    }
}

impl Into<ConfigType> for &str {
    fn into(self) -> ConfigType {
        match self {
            "usersecrets_configs" => ConfigType::UserSecrets,
            "envvars_configs" => ConfigType::EnvVars,
            _ => {
                panic!("failed to match config name to any known service implementations")
            }
        }
    }
}

// Must implement Default due to struct Config reqs
impl Default for ConfigType {
    fn default() -> Self {
        ConfigType::EnvVars
    }
}

// implements wit-bindgen's Configs for our Configs struct
impl configs::Configs for Configs {
    type Configs = ConfigsInner;

    // opens our config store dependant on the provided type
    fn configs_open(&mut self) -> Result<Self::Configs, configs::Error> {
        // set global config type
        let inner = Self::Configs {
            config_type: Arc::new(self.host_state.config_type.as_str().into()),
        };
        let rd = Uuid::new_v4().to_string();
        self.host_state
            .resource_map
            .lock()
            .unwrap()
            .set(rd, Box::new(inner.clone()));
        Ok(inner)
    }

    fn configs_get(&mut self, self_: &Self::Configs, key: &str) -> Result<Vec<u8>, configs::Error> {
        let inner = &self_.config_type;
        Ok(providers::get(
            &String::from(*inner.clone()),
            key,
            &self.host_state.config_toml_file_path,
        )?)
    }

    fn configs_set(
        &mut self,
        self_: &Self::Configs,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), configs::Error> {
        let inner = &self_.config_type;
        Ok(providers::set(
            &String::from(*inner.clone()),
            key,
            value,
            &self.host_state.config_toml_file_path,
        )?)
    }
}
