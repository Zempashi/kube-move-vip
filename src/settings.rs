use std::collections::HashMap;
use figment::{Error, Figment, providers::{Format, Yaml, Env}};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Oneprovider {
    pub api_key: String,
    pub client_key: String,
    pub nodes: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Provider {
    pub oneprovider: Oneprovider,
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Config {
    pub provider: Provider,
}

impl Config {
    pub fn new() -> Result<Self, Error> {
        let config: Config = Figment::new()
            .merge(Yaml::file("/etc/kube-move-vip/config.yaml"))
            .merge(Yaml::file("config.yaml"))
            .merge(Env::prefixed("KMV_").split("__"))
            .extract()?;
        Ok(config)
    }
}
