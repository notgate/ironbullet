use super::*;

/// Generate a random public IPv4 address.
/// Avoids private ranges (10.x, 172.16-31.x, 192.168.x, 127.x, 169.254.x)
fn random_public_ipv4() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Use current time + thread id as entropy source (no rand dep needed)
    let mut h = DefaultHasher::new();
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos()
        .hash(&mut h);
    std::thread::current().id().hash(&mut h);
    let seed = h.finish();

    // Simple LCG to get 4 octets
    let lcg = |s: u64| s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    let s1 = lcg(seed);
    let s2 = lcg(s1);
    let s3 = lcg(s2);
    let s4 = lcg(s3);

    let o1 = (s1 >> 32) as u8;
    let o2 = (s2 >> 32) as u8;
    let o3 = (s3 >> 32) as u8;
    let o4 = ((s4 >> 32) as u8).max(1); // avoid .0

    // Avoid private ranges — regenerate first octet if in private space
    let o1 = match o1 {
        0 | 10 | 127 | 169 | 172 | 192 | 198 | 203 | 240..=255 => {
            // Pick a safe first octet from common public ranges
            let safe = [1u8, 2, 4, 5, 8, 12, 14, 15, 17, 18, 20, 23, 24, 31, 34, 37,
                        38, 40, 41, 43, 44, 45, 46, 47, 50, 51, 52, 53, 54, 55, 57,
                        58, 59, 60, 61, 62, 63, 64, 66, 67, 68, 69, 70, 71, 72, 73,
                        74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88];
            safe[o2 as usize % safe.len()]
        }
        n => n,
    };

    format!("{}.{}.{}.{}", o1, o2, o3, o4)
}

impl ExecutionContext {
    pub(super) async fn execute_header_spoof(&mut self, _block: &Block, settings: &HeaderSpoofSettings) -> crate::error::Result<()> {
        // Resolve the IP to inject
        let ip = match &settings.strategy {
            IpSpoofStrategy::RandomPublic => random_public_ipv4(),
            IpSpoofStrategy::FixedList => {
                let lines: Vec<&str> = settings.fixed_ips
                    .lines()
                    .map(|l| l.trim())
                    .filter(|l| !l.is_empty())
                    .collect();
                if lines.is_empty() {
                    random_public_ipv4()
                } else {
                    // Rotate based on time so consecutive calls get different IPs
                    let idx = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as usize % lines.len();
                    lines[idx].to_string()
                }
            }
            IpSpoofStrategy::FromProxy => {
                // Strip scheme and auth from proxy URL → use host as IP
                let proxy = self.proxy.clone().unwrap_or_default();
                proxy.trim_start_matches("http://")
                     .trim_start_matches("https://")
                     .trim_start_matches("socks5://")
                     .split('@').last()
                     .unwrap_or("")
                     .split(':').next()
                     .unwrap_or(&proxy)
                     .to_string()
            }
            IpSpoofStrategy::Manual => {
                self.variables.interpolate(&settings.manual_value)
            }
        };

        // Build the header KV pairs and inject into a special variable
        // that the HTTP Request block reads to prepend headers.
        let mut injected: Vec<(String, String)> = Vec::new();

        if settings.inject_xff {
            injected.push(("X-Forwarded-For".to_string(), ip.clone()));
        }
        if settings.inject_x_real_ip {
            injected.push(("X-Real-IP".to_string(), ip.clone()));
        }
        if settings.inject_cf_connecting_ip {
            injected.push(("CF-Connecting-IP".to_string(), ip.clone()));
        }
        if settings.inject_true_client_ip {
            injected.push(("True-Client-IP".to_string(), ip.clone()));
        }
        if settings.inject_proto {
            injected.push(("X-Forwarded-Proto".to_string(), "https".to_string()));
        }
        if settings.inject_host {
            // X-Forwarded-Host gets set dynamically per-request in the HTTP block
            // by reading SPOOF_HOST. Set to empty here; HTTP block fills it in.
            injected.push(("X-Forwarded-Host".to_string(), "".to_string()));
        }

        // Store injected headers as JSON in a variable — HTTP block reads SPOOF_HEADERS
        let json = serde_json::to_string(&injected).unwrap_or_default();
        self.variables.set_user("SPOOF_HEADERS", json, true);
        self.variables.set_user("SPOOF_IP", ip.clone(), true);

        if !settings.output_var.is_empty() {
            self.variables.set_user(&settings.output_var, ip.clone(), true);
        }

        self.log.push(LogEntry {
            timestamp_ms: elapsed_ms(),
            block_id: Uuid::nil(),
            block_label: "HeaderSpoof".into(),
            message: format!("IP spoofing headers set → {} ({})", ip, format_strategy(&settings.strategy)),
        });
        Ok(())
    }
}

fn format_strategy(s: &IpSpoofStrategy) -> &'static str {
    match s {
        IpSpoofStrategy::RandomPublic => "random public",
        IpSpoofStrategy::FixedList => "fixed list",
        IpSpoofStrategy::FromProxy => "from proxy",
        IpSpoofStrategy::Manual => "manual",
    }
}
