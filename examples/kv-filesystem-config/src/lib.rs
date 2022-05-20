wit_bindgen_rust::export!("../../wit/config.wit");

/// A Filesystem configuration
#[derive(Debug, Clone)]
pub struct Config {}

impl config::Config for Config {
    /// the Filesystem configuration will have a {path: String} field.
    fn get_capability() -> Result<config::Map, config::Error> {
        let config = Vec::from([("url".to_string(), "file:///tmp".to_string())]);
        Ok(config)
    }
}
