wit_bindgen_rust::export!("../../wit/config.wit");

/// A azure blob storage configuration
#[derive(Debug, Clone)]
pub struct Config {}

impl config::Config for Config {
    /// the azure blob storage configuration
    fn get_capability() -> Result<config::Map, config::Error> {
        let config = Vec::from([("url".to_string(), "azblob://my-container".to_string())]);
        Ok(config)
    }
}
