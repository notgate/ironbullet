use super::*;

/// Generate a random public IPv4 address using the thread-local `rand` RNG.
/// Avoids all private/reserved ranges: 0.x, 10.x, 100.64-127.x (CGNAT),
/// 127.x, 169.254.x, 172.16-31.x, 192.0.x, 192.168.x, 198.18-19.x,
/// 203.0.113.x, 224+ (multicast/reserved).
fn random_public_ipv4() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // Safe first-octet set — excludes all IANA special-purpose ranges.
    // Using a curated list is simpler and more correct than rejection-sampling
    // because the private ranges cluster at well-known values.
    const SAFE_FIRST: &[u8] = &[
        1, 2, 4, 5, 8, 12, 14, 15, 17, 18, 20, 23, 24, 31, 34, 37,
        38, 40, 41, 43, 44, 45, 46, 47, 50, 51, 52, 53, 54, 55, 57,
        58, 59, 60, 61, 62, 63, 64, 66, 67, 68, 69, 70, 71, 72, 73,
        74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88,
        89, 90, 91, 93, 94, 95, 96, 97, 98, 99, 101, 102, 103, 104,
        105, 106, 107, 108, 109, 110, 111, 112, 113, 114, 115, 116,
        117, 118, 119, 120, 121, 122, 123, 124, 125, 126, 128, 129,
        130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141,
        142, 143, 144, 145, 146, 147, 148, 149, 150, 151, 152, 153,
        154, 155, 156, 157, 158, 159, 160, 161, 162, 163, 164, 165,
        166, 167, 168, 170, 171, 173, 174, 175, 176, 177, 178, 179,
        180, 181, 182, 183, 184, 185, 186, 187, 188, 189, 190, 191,
        193, 194, 195, 196, 197, 199, 200, 201, 202, 204, 205, 206,
        207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218,
        219, 220, 221, 222, 223,
    ];

    let o1 = SAFE_FIRST[rng.gen_range(0..SAFE_FIRST.len())];
    let o2: u8 = rng.gen();
    let o3: u8 = rng.gen();
    let o4: u8 = rng.gen_range(1..=254); // avoid .0 and .255

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
