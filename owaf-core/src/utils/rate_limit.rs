use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RateLimiter {
    // Maps (Host, IP) -> (WindowStartSec, Count)
    store: Mutex<HashMap<(String, String), (u64, u64)>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            store: Mutex::new(HashMap::new()),
        }
    }

    pub fn check(&self, host: &str, ip: &str, limit_reqs: u64, window_sec: u64) -> bool {
        if limit_reqs == 0 {
            return false;
        }
        if window_sec == 0 {
            return true;
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let current_window = now / window_sec;
        
        let mut store = self.store.lock().unwrap();
        
        // Prevent unbounded growth
        if store.len() > 10000 {
            store.retain(|_, v| v.0 >= current_window.saturating_sub(1));
        }

        let key = (host.to_string(), ip.to_string());
        let entry = store.entry(key).or_insert((current_window, 0));

        if entry.0 != current_window {
            entry.0 = current_window;
            entry.1 = 1;
            true
        } else {
            if entry.1 < limit_reqs {
                entry.1 += 1;
                true
            } else {
                false
            }
        }
    }
}

pub fn get_limiter() -> &'static RateLimiter {
    static LIMITER: OnceLock<RateLimiter> = OnceLock::new();
    LIMITER.get_or_init(RateLimiter::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new();
        let host = "example.com";
        let ip = "192.168.1.1";

        // Limit: 2 requests per 10 seconds
        assert!(limiter.check(host, ip, 2, 10)); // 1st req
        assert!(limiter.check(host, ip, 2, 10)); // 2nd req
        assert!(!limiter.check(host, ip, 2, 10)); // 3rd req - blocked
        
        // Different IP should be allowed
        let ip2 = "10.0.0.1";
        assert!(limiter.check(host, ip2, 2, 10)); // 1st req
        
        // Zero limit blocks all
        assert!(!limiter.check(host, ip, 0, 10));
        
        // Zero window allows all
        assert!(limiter.check(host, ip, 2, 0));
    }
}
