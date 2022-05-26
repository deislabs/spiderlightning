wit_bindgen_rust::export!("../../wit/config.wit");

/// A mq (azure servicebus) configuration
#[derive(Debug, Clone)]
pub struct Config {}

impl config::Config for Config {
    /// the mq (azure servicebus) configuration
    fn get_capability() -> Result<config::Map, config::Error> {
        let config = Vec::from([("url".to_string(), "azmq://tmp".to_string())]);
        Ok(config)
    }
}
