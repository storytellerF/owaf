use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize, Clone, Debug, Default)]
pub struct ProxyConfig {
    #[serde(default)]
    pub proxy: HashMap<String, ProxyEntry>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RateLimitConfig {
    pub requests: u64,
    pub window_sec: u64,
}

#[derive(Deserialize, Clone, Debug)]
pub struct ProxyEntry {
    pub host: String,
    pub target: String,
    #[serde(default)]
    pub rate_limit: Option<RateLimitConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rate_limit_hcl() {
        let hcl_str = r#"
            proxy "test" {
                host = "example.com"
                target = "http://localhost:8080"
                rate_limit {
                    requests = 100
                    window_sec = 60
                }
            }
        "#;
        
        let config: ProxyConfig = hcl::from_str(hcl_str).unwrap();
        let entry = config.proxy.get("test").unwrap();
        
        assert_eq!(entry.host, "example.com");
        let rl = entry.rate_limit.as_ref().unwrap();
        assert_eq!(rl.requests, 100);
        assert_eq!(rl.window_sec, 60);
    }
}
