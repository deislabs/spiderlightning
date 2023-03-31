pub mod implementors;

use std::{collections::HashMap, path::Path};

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use regex::Regex;

use implementors::{azapp::AzApp, envvars::EnvVars, usersecrets::UserSecrets};
use slight_common::{impl_resource, BasicState};

wit_bindgen_wasmtime::export!({paths: ["../../wit/configs.wit"], async: *});
wit_error_rs::impl_error!(configs::ConfigsError);
wit_error_rs::impl_from!(anyhow::Error, configs::ConfigsError::UnexpectedError);

/// The `Configs` structure is what will implement the `configs::Configs` trait
/// coming from the generated code of off `configs.wit`.
///
/// It holds:
///     - an `implementor` `String` — this comes directly from a
///     user's `slightfile` and it is what allows us to dynamically
///     dispatch to a specific implementor's implentation, and
///     - the `slight_state` (of type `BasicState`) that contains common
///     things received from the slight binary (i.e., the `config_type`
///     and the `slightfile_path`).
#[derive(Clone, Default)]
pub struct Configs {
    implementor: String,
    capability_store: HashMap<String, BasicState>,
}

impl Configs {
    pub fn new(implementor: String, capability_store: HashMap<String, BasicState>) -> Self {
        Self {
            implementor,
            capability_store,
        }
    }
}

impl_resource!(
    Configs,
    configs::ConfigsTables<Configs>,
    configs::add_to_linker,
    "configs".to_string()
);

#[async_trait]
impl configs::Configs for Configs {
    type Configs = ConfigsInner;

    async fn configs_open(&mut self, name: &str) -> Result<Self::Configs, configs::ConfigsError> {
        // populate our inner configs object w/ the state received from `slight`
        // (i.e., what type of configs implementor we are using), and the assigned
        // name of the object.
        let state = if let Some(r) = self.capability_store.get(name) {
            r.clone()
        } else if let Some(r) = self.capability_store.get(&self.implementor) {
            r.clone()
        } else {
            panic!(
                "could not find capability under name '{}' for implementor '{}'",
                name, &self.implementor
            );
        };

        tracing::log::info!("Opening implementor {}", &state.implementor);

        let inner = Self::Configs::new(&state.implementor, &state);

        Ok(inner)
    }

    async fn configs_get(
        &mut self,
        self_: &Self::Configs,
        key: &str,
    ) -> Result<Vec<u8>, configs::ConfigsError> {
        Ok(get(
            &String::from(&self_.configs_implementor),
            key,
            &self_.slight_state.slightfile_path,
        )
        .await?)
    }

    async fn configs_set(
        &mut self,
        self_: &Self::Configs,
        key: &str,
        value: &[u8],
    ) -> Result<(), configs::ConfigsError> {
        set(
            &String::from(&self_.configs_implementor),
            key,
            value,
            &self_.slight_state.slightfile_path,
        )
        .await?;

        Ok(())
    }
}

/// This is the type of the associated type coming from the `configs::Configs` trait
/// implementation.
///
/// It holds:
///     - a `configs_implementor` (i.e., a variant `ConfigsImplementor` `enum`), and
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
    slight_state: BasicState,
}

impl ConfigsInner {
    fn new(configs_implementor: &str, slight_state: &BasicState) -> Self {
        Self {
            configs_implementor: configs_implementor.into(),
            slight_state: slight_state.clone(),
        }
    }
}

/// This defines the available implementor implementations for the `Configs` interface.
///
/// As per its' usage in `ConfigsInner`, it must `derive` `Debug`, and `Clone`.
#[derive(Debug, Clone, Default)]
pub enum ConfigsImplementor {
    Local,
    #[default]
    EnvVars,
    UserSecrets, // user creates configs at compile time that are encrypted and stored in their slightfile
    AzApp,
}

impl From<&ConfigsImplementor> for String {
    fn from(c: &ConfigsImplementor) -> String {
        match c {
            ConfigsImplementor::UserSecrets => "configs.usersecrets".to_string(),
            ConfigsImplementor::EnvVars => "configs.envvars".to_string(),
            ConfigsImplementor::AzApp => "configs.azapp".to_string(),
            _ => panic!("unknown configuration type"),
        }
    }
}

impl From<&str> for ConfigsImplementor {
    fn from(from_str: &str) -> Self {
        match from_str {
            "configs.usersecrets" => ConfigsImplementor::UserSecrets,
            "configs.envvars" => ConfigsImplementor::EnvVars,
            "configs.azapp" => ConfigsImplementor::AzApp,
            "configs.local" => ConfigsImplementor::Local,
            _ => panic!("unknown configuration type '{from_str}'"),
        }
    }
}

/// SDK-ish bit
pub async fn get(
    config_type: &str,
    key: &str,
    toml_file_path: impl AsRef<Path>,
) -> Result<Vec<u8>> {
    match config_type.into() {
        ConfigsImplementor::EnvVars => Ok(EnvVars::get(key)?),
        ConfigsImplementor::UserSecrets => Ok(UserSecrets::get(key, toml_file_path)?),
        ConfigsImplementor::AzApp => Ok(AzApp::get(key).await?),
        ConfigsImplementor::Local => Ok(key.as_bytes().to_vec()),
    }
}

pub async fn set(
    config_type: &str,
    key: &str,
    value: &[u8],
    toml_file_path: impl AsRef<Path>,
) -> Result<()> {
    match config_type.into() {
        ConfigsImplementor::EnvVars => Ok(EnvVars::set(key, value)?),
        ConfigsImplementor::UserSecrets => Ok(UserSecrets::set(key, value, toml_file_path)?),
        ConfigsImplementor::AzApp => Ok(AzApp::set(key, value).await?),
        _ => bail!("unknown configuration type"),
    }
}

pub async fn get_from_state(config_name: &str, state: &BasicState) -> Result<String> {
    if let Some(ss) = &state.secret_store {
        let config = String::from_utf8(
            get(ss, config_name, &state.slightfile_path)
                .await
                .with_context(|| {
                    format!("failed to get '{config_name}' secret using secret store type: {ss}")
                })?,
        )?;
        Ok(config)
    } else {
        let c = state
            .configs_map
            .as_ref()
            .expect("this capability needs a [capability.configs] section...")
            .get(config_name)
            .with_context(|| format!("no config named '{config_name}' found"))?;

        let (store, name) = maybe_get_config_store_and_value(c)?;

        let config = String::from_utf8(
            get(&store, &name, &state.slightfile_path)
                .await
                .with_context(|| {
                    format!(
                        "failed to get '{config_name}' secret using secret store type: '{store}'"
                    )
                })?,
        )?;
        Ok(config)
    }
}

fn maybe_get_config_store_and_value(c: &str) -> Result<(String, String)> {
    let mut regex_match = Regex::new(r"^\$\{(.+)\}$")?;
    if let Some(prelim_cap) = regex_match.captures(c) {
        regex_match = Regex::new(r"(.+)\.(.+)")?;
        if let Some(cap) = regex_match.captures(&prelim_cap[1]) {
            Ok((format!("configs.{}", &cap[1]), cap[2].to_string()))
        } else {
            panic!("failed to get value for config '{c}'");
        }
    } else {
        Ok(("configs.local".to_string(), c.to_string()))
    }
}

#[cfg(test)]
mod unittests {
    use anyhow::Result;
    use slight_core::slightfile::TomlFile;

    use crate::maybe_get_config_store_and_value;

    #[test]
    fn parse_this_dot_that() -> Result<()> {
        let toml_file_contents = r#"
        specversion = "0.1"
        [[capability]]
        resource = "keyvalue.azblob"
        name = "customers"
            [capability.configs]
            a = "${azapp.hello}"
        "#;
        let toml = toml::from_str::<TomlFile>(toml_file_contents)?;
        assert_eq!(
            ("configs.azapp".to_string(), "hello".to_string()),
            maybe_get_config_store_and_value(
                toml.capability.as_ref().unwrap()[0]
                    .configs
                    .as_ref()
                    .unwrap()
                    .get("a")
                    .unwrap(),
            )?
        );

        Ok(())
    }

    #[test]
    #[should_panic]
    fn parse_this_missing_dot_that() {
        let toml_file_contents = r#"
        specversion = "0.1"
        [[capability]]
        resource = "keyvalue.azblob"
        name = "customers"
            [capability.configs]
            b = "${cruel}"
        "#;
        let toml = toml::from_str::<TomlFile>(toml_file_contents).unwrap();
        maybe_get_config_store_and_value(
            toml.capability.as_ref().unwrap()[0]
                .configs
                .as_ref()
                .unwrap()
                .get("b")
                .unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn parse_local_config() -> Result<()> {
        let toml_file_contents = r#"
        specversion = "0.1"
        [[capability]]
        resource = "keyvalue.azblob"
        name = "customers"
            [capability.configs]
            c = "world"
        "#;
        let toml = toml::from_str::<TomlFile>(toml_file_contents)?;
        assert_eq!(
            ("configs.local".to_string(), "world".to_string()),
            maybe_get_config_store_and_value(
                toml.capability.as_ref().unwrap()[0]
                    .configs
                    .as_ref()
                    .unwrap()
                    .get("c")
                    .unwrap(),
            )?
        );

        Ok(())
    }
}
