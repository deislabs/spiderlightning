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

mod providers;

const SCHEME_NAME: &str = "configs";

// Struct Representer for wit_bindgen's Config
#[derive(Default, Clone, Resource)]
struct Configs {
    inner: Option<Arc<ConfigType>>, // have to wrap it in Option<Arc<>> due to Resource derive proc macro
    host_state: Option<ResourceMap>,
}

impl_resource!(
    Configs,
    configs::ConfigsTables<Configs>,
    ResourceMap,
    SCHEME_NAME.to_string()
);

// Currently supported configuration types
enum ConfigType {
    Local,       // user creates configs in plain text at runtime
    UserSecrets, // user creates configs at compile time that are encrypted and stored in the toml file
}

impl ConfigType {
    fn new(what: ConfigType) -> Option<Arc<ConfigType>> {
        Some(Arc::new(what))
    }
}

// Must implement Default due to struct Config reqs
impl Default for ConfigType {
    fn default() -> Self {
        ConfigType::Local
    }
}

// implements wit-bindgen's Configs for our Configs struct
impl configs::Configs for Configs {
    type Configs = String;

    // opens our config store dependant on the provided type
    fn configs_open(&mut self, name: &str) -> Result<Self::Configs, Error> {
        // set global config type
        self.inner = match name {
            "usersecrets" => ConfigType::new(ConfigType::UserSecrets),
            "local" => ConfigType::new(ConfigType::Local),
            _ => {
                return Err(configs::Error::ErrorWithDescription(
                    "failed to match config name to any known service implementations".to_string(),
                ))
            }
        };

        let rd = Uuid::new_v4().to_string();
        let cloned = self.clone(); // have to clone here because of the mutable borrow below
        let mut map = Map::lock(&mut self.host_state)?;
        map.set(rd.clone(), (Box::new(cloned), None));

        Ok(rd)
    }

    fn configs_get(&mut self, self_: &Self::Configs, key: &str) -> Result<Vec<u8>, Error> {
        Uuid::parse_str(self_).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::lock(&mut self.host_state)?;
        let inner = map.get::<Arc<ConfigType>>(self_)?;

        match *inner.clone() {
            ConfigType::Local => todo!("local get is still not implemented"),
            ConfigType::UserSecrets => Ok(providers::usersecrets::get_config_usersecrets(key)?),
        }
    }

    fn configs_set(
        &mut self,
        self_: &Self::Configs,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        Uuid::parse_str(self_).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::lock(&mut self.host_state)?;
        let inner = map.get::<Arc<ConfigType>>(self_)?;

        match *inner.clone() {
            ConfigType::Local => todo!("local set is still not implemented"),
            ConfigType::UserSecrets => Ok(providers::usersecrets::set_config_usersecrets(key, value)?)
        }
    }
}
