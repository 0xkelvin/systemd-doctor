use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub services: Services,
    // pub thresholds: HashMap<String, ServiceThreshold>,
    // pub global_thresholds: GlobalThresholds,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Services {
    pub list: Option<Vec<String>>,
}

#[derive(Clone, Debug)]
pub struct ConfigParser {
    pub config: Config,
}

impl ConfigParser {
    pub fn new(config_path: &str) -> Self {
        let config_contents = fs::read_to_string(config_path).expect("Failed to read config file");
        let config: Config = toml::from_str(&config_contents).expect("Failed to parse config file");
        ConfigParser { config }
    }

    pub fn get_config(&self) -> &Config {
        &self.config
    }

    pub fn get_config_services(&self) -> &Option<Vec<String>> {
        &self.config.services.list
    }
}
