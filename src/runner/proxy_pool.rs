use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

pub struct ProxyPool {
    proxies: Vec<ProxyEntry>,
    index: AtomicUsize,
    bans: Mutex<HashMap<String, Instant>>,
    ban_duration_secs: u64,
}

#[derive(Debug, Clone)]
pub struct ProxyEntry {
    pub proxy_type: ProxyType,
    pub address: String,
}

#[derive(Debug, Clone, Copy)]
pub enum ProxyType {
    Http,
    Https,
    Socks4,
    Socks5,
}

impl ProxyPool {
    pub fn new(proxies: Vec<ProxyEntry>, ban_duration_secs: u64) -> Self {
        Self {
            proxies,
            index: AtomicUsize::new(0),
            bans: Mutex::new(HashMap::new()),
            ban_duration_secs,
        }
    }

    pub fn from_file(path: &str, ban_duration_secs: u64) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let proxies: Vec<ProxyEntry> = content.lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|l| parse_proxy_line(l.trim()))
            .collect();
        Ok(Self::new(proxies, ban_duration_secs))
    }

    pub fn empty() -> Self {
        Self::new(Vec::new(), 300)
    }

    pub fn next_proxy(&self) -> Option<String> {
        if self.proxies.is_empty() {
            return None;
        }

        let now = Instant::now();

        // Acquire ban-list lock; if poisoned, recover and continue (never block callers)
        let bans = match self.bans.lock() {
            Ok(g)  => g,
            Err(e) => e.into_inner(), // poisoned — recover the data, don't return None
        };

        // Try to find an unbanned proxy (scan at most proxies.len() candidates)
        let start = self.index.fetch_add(1, Ordering::Relaxed);
        for i in 0..self.proxies.len() {
            let idx = (start + i) % self.proxies.len();
            let proxy = &self.proxies[idx];
            let proxy_str = proxy.to_string();

            match bans.get(&proxy_str) {
                Some(ban_time) if now.duration_since(*ban_time).as_secs() < self.ban_duration_secs => {
                    // Still banned — skip
                }
                _ => return Some(proxy_str),
            }
        }

        // All proxies banned — return the round-robin slot anyway so work continues
        // (better to hit a banned proxy than to stall all threads with None)
        let idx = start % self.proxies.len();
        Some(self.proxies[idx].to_string())
    }

    pub fn ban_proxy(&self, proxy: &str) {
        if let Ok(mut bans) = self.bans.lock() {
            bans.insert(proxy.to_string(), Instant::now());
        }
    }

    pub fn total(&self) -> usize {
        self.proxies.len()
    }

    pub fn active(&self) -> usize {
        let bans = self.bans.lock().ok();
        let now = Instant::now();
        let banned = bans.map(|b| {
            b.values().filter(|t| now.duration_since(**t).as_secs() < self.ban_duration_secs).count()
        }).unwrap_or(0);
        self.proxies.len().saturating_sub(banned)
    }
}

impl std::fmt::Display for ProxyEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.proxy_type {
            ProxyType::Http => "http://",
            ProxyType::Https => "https://",
            ProxyType::Socks4 => "socks4://",
            ProxyType::Socks5 => "socks5://",
        };
        write!(f, "{}{}", prefix, self.address)
    }
}

fn parse_proxy_line(line: &str) -> Option<ProxyEntry> {
    // Formats: TYPE:HOST:PORT:USER:PASS, HOST:PORT, protocol://HOST:PORT
    if line.starts_with("http://") || line.starts_with("https://") ||
       line.starts_with("socks4://") || line.starts_with("socks5://") {
        let (proxy_type, rest) = if let Some(rest) = line.strip_prefix("socks5://") {
            (ProxyType::Socks5, rest)
        } else if let Some(rest) = line.strip_prefix("socks4://") {
            (ProxyType::Socks4, rest)
        } else if let Some(rest) = line.strip_prefix("https://") {
            (ProxyType::Https, rest)
        } else {
            (ProxyType::Http, line.strip_prefix("http://").unwrap_or(line))
        };
        return Some(ProxyEntry { proxy_type, address: rest.to_string() });
    }

    let parts: Vec<&str> = line.split(':').collect();
    match parts.len() {
        2 => Some(ProxyEntry {
            proxy_type: ProxyType::Http,
            address: format!("{}:{}", parts[0], parts[1]),
        }),
        4 => Some(ProxyEntry {
            proxy_type: ProxyType::Http,
            address: format!("{}:{}@{}:{}", parts[2], parts[3], parts[0], parts[1]),
        }),
        5 => {
            let proxy_type = match parts[0].to_lowercase().as_str() {
                "http" => ProxyType::Http,
                "https" => ProxyType::Https,
                "socks4" => ProxyType::Socks4,
                "socks5" => ProxyType::Socks5,
                _ => ProxyType::Http,
            };
            Some(ProxyEntry {
                proxy_type,
                address: format!("{}:{}@{}:{}", parts[3], parts[4], parts[1], parts[2]),
            })
        }
        _ => None,
    }
}
