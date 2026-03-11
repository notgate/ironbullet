//! Embedded Shadowsocks local-proxy pool.
//!
//! For each unique SS server in the proxy list (`ss://method:password@host:port`),
//! we spin up a shadowsocks-service local SOCKS5 listener on a random free port
//! on 127.0.0.1. The listener is kept alive for the duration of the process.
//!
//! When `resolve(ss_url)` is called, it returns `socks5://127.0.0.1:<port>` which
//! the HTTP clients (reqwest/wreq/azuretls) can use directly. Subsequent calls for
//! the same SS server reuse the existing listener — no per-request overhead.
//!
//! Supported cipher formats (PIA and most providers use aes-128-gcm):
//!   - aes-128-gcm, aes-256-gcm, chacha20-ietf-poly1305
//!   - 2022 AEAD (aes-128-gcm-2022, aes-256-gcm-2022, etc.) with `aead-cipher-2022` feature
//!
//! URI format accepted:
//!   ss://method:password@host:port
//!   ss://BASE64(method:password)@host:port  (SIP002)

use std::{
    collections::HashMap,
    net::{SocketAddr, TcpListener},
    sync::Mutex,
};

use shadowsocks::{
    config::{Mode, ServerConfig},
    crypto::CipherKind,
    ServerAddr,
};
use shadowsocks_service::{
    config::{Config, ConfigType, LocalConfig, LocalInstanceConfig, ProtocolType, ServerInstanceConfig},
    local::Server,
};

/// Global singleton pool — initialised once per process.
static POOL: std::sync::OnceLock<ShadowsocksPool> = std::sync::OnceLock::new();

pub struct ShadowsocksPool {
    /// Maps canonical ss:// URL → local `socks5://127.0.0.1:<port>` string.
    map: Mutex<HashMap<String, String>>,
}

impl ShadowsocksPool {
    fn new() -> Self {
        Self { map: Mutex::new(HashMap::new()) }
    }

    /// Get-or-create the local SOCKS5 proxy URL for a given `ss://` URI.
    /// Returns the `socks5://127.0.0.1:<port>` string on success.
    pub fn resolve(&self, ss_url: &str) -> Result<String, String> {
        // Fast path — already started
        {
            let guard = self.map.lock().unwrap();
            if let Some(local) = guard.get(ss_url) {
                return Ok(local.clone());
            }
        }

        // Parse the ss:// URI
        let parsed = parse_ss_uri(ss_url)?;

        // Allocate a free port
        let port = free_port().ok_or("no free local port available")?;
        let listen_addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
        let local_url = format!("socks5://127.0.0.1:{}", port);

        // Build shadowsocks Config
        let ss_server_addr = ServerAddr::from((parsed.host.as_str(), parsed.port));
        let svr_cfg = ServerConfig::new(
            ss_server_addr,
            parsed.password.clone(),
            parsed.method,
        ).map_err(|e| format!("bad SS server config: {e}"))?;

        let mut config = Config::new(ConfigType::Local);
        config.server.push(ServerInstanceConfig::with_server_config(svr_cfg));

        let mut local_cfg = LocalConfig::new(ProtocolType::Socks);
        local_cfg.addr = Some(ServerAddr::SocketAddr(listen_addr));
        local_cfg.mode = Mode::TcpOnly;
        config.local.push(LocalInstanceConfig::with_local_config(local_cfg));

        // Spawn the local SS server on the tokio runtime (fire-and-forget).
        // `Server::new` + `run` are async, so we spawn a task.
        tokio::spawn(async move {
            match Server::new(config).await {
                Ok(server) => {
                    if let Err(e) = server.run().await {
                        eprintln!("[ss-pool] server for port {} exited: {e}", port);
                    }
                }
                Err(e) => {
                    eprintln!("[ss-pool] failed to start local SS server on port {port}: {e}");
                }
            }
        });

        let mut guard = self.map.lock().unwrap();
        // Double-check in case another thread raced us
        if let Some(existing) = guard.get(ss_url) {
            return Ok(existing.clone());
        }
        guard.insert(ss_url.to_string(), local_url.clone());
        Ok(local_url)
    }
}

/// Get or initialise the global pool.
pub fn pool() -> &'static ShadowsocksPool {
    POOL.get_or_init(ShadowsocksPool::new)
}

/// Resolve a `ss://` URI to a local `socks5://127.0.0.1:<port>` string.
/// Returns the original URL on parse failure so the caller can attempt it anyway.
pub fn resolve_ss_proxy(ss_url: &str) -> String {
    match pool().resolve(ss_url) {
        Ok(local) => local,
        Err(e) => {
            eprintln!("[ss-pool] failed to resolve {ss_url}: {e}");
            ss_url.to_string() // pass through — will fail gracefully at the HTTP layer
        }
    }
}

/// Reverse lookup: given a `socks5://127.0.0.1:<port>` tunnel URL, return
/// the original `ss://...` key if it was created by this pool.
/// Returns `None` if the URL doesn't correspond to a known SS tunnel.
pub fn canonical_for_local(local_url: &str) -> Option<String> {
    let guard = pool().map.lock().unwrap();
    // Reverse scan — pool is small (one entry per unique SS server)
    for (canonical, local) in guard.iter() {
        if local == local_url {
            return Some(canonical.clone());
        }
    }
    None
}

// ── URI parsing ─────────────────────────────────────────────────────────────

struct ParsedSsUri {
    method:   CipherKind,
    password: String,
    host:     String,
    port:     u16,
}

/// Parse `ss://method:password@host:port` or `ss://BASE64@host:port` (SIP002).
fn parse_ss_uri(uri: &str) -> Result<ParsedSsUri, String> {
    // Strip the `ss://` prefix (caller should already have it)
    let rest = uri.strip_prefix("ss://").unwrap_or(uri);

    // Split on the last '@' to get userinfo@hostinfo
    let at = rest.rfind('@').ok_or_else(|| format!("missing '@' in ss URI: {uri}"))?;
    let userinfo = &rest[..at];
    let hostinfo = &rest[at + 1..];

    // Parse host:port
    let (host, port) = if let Some(colon) = hostinfo.rfind(':') {
        let h = hostinfo[..colon].to_string();
        let p: u16 = hostinfo[colon + 1..].parse()
            .map_err(|_| format!("invalid port in ss URI: {uri}"))?;
        (h, p)
    } else {
        return Err(format!("missing port in ss URI: {uri}"));
    };

    // Parse method:password (or BASE64-encoded method:password per SIP002)
    let (method_str, password) = if let Some(colon) = userinfo.find(':') {
        // Cleartext: method:password
        (userinfo[..colon].to_string(), userinfo[colon + 1..].to_string())
    } else {
        // Try BASE64 decode (SIP002: base64url(method:password))
        let decoded = base64_decode(userinfo)
            .ok_or_else(|| format!("cannot decode userinfo in ss URI: {uri}"))?;
        if let Some(colon) = decoded.find(':') {
            (decoded[..colon].to_string(), decoded[colon + 1..].to_string())
        } else {
            return Err(format!("decoded ss userinfo has no ':': {decoded}"));
        }
    };

    let method: CipherKind = method_str.parse()
        .map_err(|_| format!("unknown SS cipher '{}' — supported: aes-128-gcm, aes-256-gcm, chacha20-ietf-poly1305, aes-128-gcm-2022, ...", method_str))?;

    Ok(ParsedSsUri { method, password, host, port })
}

fn base64_decode(s: &str) -> Option<String> {
    // Try standard and URL-safe base64
    let padded = {
        let mut p = s.to_string();
        while p.len() % 4 != 0 { p.push('='); }
        p
    };
    // Simple base64 decode using std only (no dep) — map url-safe chars first
    let standard = padded.replace('-', "+").replace('_', "/");
    base64_decode_standard(&standard)
}

fn base64_decode_standard(s: &str) -> Option<String> {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut map = [255u8; 256];
    for (i, &b) in TABLE.iter().enumerate() {
        map[b as usize] = i as u8;
    }
    let s = s.trim_end_matches('=');
    let mut out = Vec::with_capacity(s.len() * 3 / 4);
    let mut buf = 0u32;
    let mut bits = 0u32;
    for c in s.bytes() {
        let val = map[c as usize];
        if val == 255 { return None; }
        buf = (buf << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buf >> bits) as u8);
        }
    }
    String::from_utf8(out).ok()
}

// ── Port allocation ──────────────────────────────────────────────────────────

/// Find a free TCP port on 127.0.0.1 by binding port 0 and reading the OS assignment.
fn free_port() -> Option<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").ok()?;
    let port = listener.local_addr().ok()?.port();
    drop(listener); // release — small TOCTOU window but acceptable for local use
    Some(port)
}
