use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct ProxyConfig {
    #[serde(default)]
    pub proxy: HashMap<String, ProxyEntry>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ProxyEntry {
    pub host: String,
    pub target: String,
}
