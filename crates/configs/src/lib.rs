use anyhow::{Context, Result};
use configs::add_to_linker;
use configs::*;
use crossbeam_channel::Sender;
use proc_macro_utils::{Resource, RuntimeResource};
use runtime::resource::{
    get, Ctx, DataT, Event, Linker, Map, Resource, ResourceMap, RuntimeResource,
};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
wit_bindgen_wasmtime::export!("../../wit/configs.wit");
wit_error_rs::impl_error!(configs::Error);
wit_error_rs::impl_from!(anyhow::Error, configs::Error::ErrorWithDescription);

mod providers;

const SCHEME_NAME: &str = "configs";

// Struct Representer for wit_bindgen's Config
#[derive(Default, Clone, Resource, RuntimeResource)]
struct Configs {
    inner: Option<Arc<ConfigType>>, // have to wrap it in Option<Arc<>> due to Resource derive proc macro
    resource_map: Option<ResourceMap>,
}

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
    // initiates our config store dependant on the provided type
    fn init_configs(&mut self, name: &str) -> Result<ResourceDescriptorResult, Error> {
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
        let mut map = Map::lock(&mut self.resource_map)?;
        map.set(rd.clone(), (Box::new(cloned), None));

        Ok(rd)
    }

    fn get_config(
        &mut self,
        rd: ResourceDescriptorParam<'_>,
        key: &str,
    ) -> Result<PayloadResult, Error> {
        Uuid::parse_str(rd).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::lock(&mut self.resource_map)?;
        let inner = map.get::<Arc<ConfigType>>(rd)?;

        match *inner.clone() {
            ConfigType::Local => todo!("local get is still not implemented"),
            ConfigType::UserSecrets => Ok(providers::usersecrets::get_config_usersecrets(key)?),
            _ => {
                return Err(configs::Error::ErrorWithDescription(
                    "failed to match config name to any known service implementations".to_string(),
                ))
            }
        }
    }

    fn set_config(
        &mut self,
        rd: ResourceDescriptorParam<'_>,
        key: &str,
        value: PayloadParam<'_>,
    ) -> Result<(), Error> {
        Uuid::parse_str(rd).with_context(|| "failed to parse resource descriptor")?;

        let map = Map::lock(&mut self.resource_map)?;
        let inner = map.get::<Arc<ConfigType>>(rd)?;

        match *inner.clone() {
            ConfigType::Local => todo!("local set is still not implemented"),
            ConfigType::UserSecrets => {
                Ok(providers::usersecrets::set_config_usersecrets(key, value)?)
            }
            _ => {
                return Err(configs::Error::ErrorWithDescription(
                    "failed to match config name to any known service implementations".to_string(),
                ))
            }
        }
    }
}
