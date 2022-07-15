use anyhow::{Context, Result};
use configs::*;
use crossbeam_channel::Sender;
use events_api::Event;
use proc_macro_utils::Resource;
use runtime::impl_resource;
use runtime::resource::{
    get_table, Ctx, DataT, Linker, Map, Resource, ResourceMap, ResourceTables, RuntimeResource,
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
    inner: Option<Arc<ConfigType>>, // have to wrap it in Option<Arc<>> due to Resource derive proc macro
    host_state: Option<ConfigsState>,
}

#[derive(Clone)]
pub struct ConfigsState {
    pub resource_map: Option<ResourceMap>,
    pub config_toml_file_path: String,
}

impl ConfigsState {
    pub fn new(resource_map: ResourceMap, config_toml_file_path: &str) -> Self {
        Self {
            resource_map: Some(resource_map),
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
#[derive(Clone, Copy)]
pub enum ConfigType {
    EnvVars,
    UserSecrets, // user creates configs at compile time that are encrypted and stored in the toml file
}

impl From<ConfigType> for String {
    fn from(from_ct: ConfigType) -> Self {
        match from_ct {
            ConfigType::UserSecrets => "usersecrets".to_string(),
            ConfigType::EnvVars => "envvars".to_string(),
        }
    }
}

impl Into<ConfigType> for &str {
    fn into(self) -> ConfigType {
        match self {
            "usersecrets" => ConfigType::UserSecrets,
            "envvars" => ConfigType::EnvVars,
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
    type Configs = String;

    // opens our config store dependant on the provided type
    fn configs_open(&mut self, name: &str) -> Result<Self::Configs, configs::Error> {
        // set global config type
        self.inner = Some(Arc::new(name.into()));
        let rd = Uuid::new_v4().to_string();
        let cloned = self.clone(); // have to clone here because of the mutable borrow below
        let mut map = Map::lock(&mut self.host_state.as_mut().unwrap().resource_map)?;
        map.set(rd.clone(), (Box::new(cloned), None));

        Ok(rd)
    }

    fn configs_get(&mut self, self_: &Self::Configs, key: &str) -> Result<Vec<u8>, configs::Error> {
        Uuid::parse_str(self_).with_context(|| "failed to parse resource descriptor")?;

        let mut mut_host_state = self.clone().host_state.unwrap();
        let map = Map::lock(&mut mut_host_state.resource_map)?;
        let inner = map.get::<Arc<ConfigType>>(self_)?;
        Ok(providers::get(
            &String::from(*inner.clone()),
            key,
            &self.host_state.as_ref().unwrap().config_toml_file_path,
        )?)
    }

    fn configs_set(
        &mut self,
        self_: &Self::Configs,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), configs::Error> {
        Uuid::parse_str(self_).with_context(|| "failed to parse resource descriptor")?;

        let mut mut_host_state = self.clone().host_state.unwrap();
        let map = Map::lock(&mut mut_host_state.resource_map)?;
        let inner = map.get::<Arc<ConfigType>>(self_)?;
        Ok(providers::set(
            &String::from(*inner.clone()),
            key,
            value,
            &self.host_state.as_ref().unwrap().config_toml_file_path,
        )?)
    }
}
