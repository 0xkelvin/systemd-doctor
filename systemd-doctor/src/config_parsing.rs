use std::fs;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub services: Services,
    pub thresholds: HashMap<String, ServiceThreshold>,
    pub global_thresholds: GlobalThresholds,
}

#[derive(Deserialize, Debug)]
pub struct Services {
    pub list: Vec<String>,
}


#[derive(Deserialize, Debug)]
pub struct ServiceThreshold {
    pub cpu: Option<f32>,
    pub memory: Option<f32>,
    pub disk: Option<f32>,
}

#[derive(Deserialize, Debug)]
pub struct GlobalThresholds {
    pub cpu: Option<f32>,
    pub memory: Option<f32>,
    pub disk: Option<f32>,
}

pub fn load_config(config_path: &str) -> Config {
    let config_str = fs::read_to_string(config_path).expect("Failed to read config file");
    toml::from_str(&config_str).expect("Failed to parse config file")
}
