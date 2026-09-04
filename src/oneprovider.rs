use std::collections::HashMap;
use reqwest;
use tracing::*;

use crate::settings;

pub struct OneproviderConfig {
  pub api_key: String,
  pub client_key: String,
  pub node_id_map: HashMap<String, OneproviderServer>
}


pub struct Oneprovider {
  pub config: OneproviderConfig,
}


pub struct OneproviderServer {
    pub server_id: String,
}

impl OneproviderConfig {

    pub fn new(settings: settings::Oneprovider) -> Self {
        Self {
            api_key: settings.api_key,
            client_key: settings.client_key,
            node_id_map: OneproviderConfig::generate_nodes(settings.nodes),
        }
    }

    pub fn generate_nodes(nodes: Option<HashMap<String, String>>) -> HashMap<String, OneproviderServer> {
        let mut node_map = HashMap::new();
        for (node, server_id) in &nodes.unwrap() {
            node_map.insert(node.to_string(), OneproviderServer{ server_id: server_id.to_string() });
        }
        node_map
    }

}

impl Oneprovider {
    pub fn new(settings: settings::Oneprovider) -> Self {
        Self {
            config: OneproviderConfig::new(settings), }
    }
}


impl Oneprovider {
   pub async fn failover(&self, failover_ip: &str, server: &str) -> Result<() ,reqwest::Error> {
        let server_id = self.config.node_id_map.get(server).unwrap().server_id.as_str();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(reqwest::header::HeaderName::from_static("api-key"), self.config.api_key.parse().unwrap());
        headers.insert(reqwest::header::HeaderName::from_static("client-key"), self.config.client_key.parse().unwrap());
        let params = [("action", "toggle_failover"), ("server_id", server_id), ("source", failover_ip)];
        let client = reqwest::Client::new();
        let resp = client.post("https://api.oneprovider.com/server/action/")
            .form(&params)
            .headers(headers)
            .send()
            .await?;
        info!("{resp:#?}");
        Ok(())
    }
}
