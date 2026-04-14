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
//! ## Fix for issue #43 — grey screen freeze after minutes of Shadowsocks use
//!
//! **Root cause 1 — `block_in_place` while holding the AppState lock:**
//! `resolve()` used `tokio::task::block_in_place` to wait for the SS listener to bind.
//! `resolve_ss_proxy` is called from `build_proxy_pool` → `start_job`, which runs
//! while the `AppState` tokio `Mutex` is held. `block_in_place` parks the calling
//! tokio worker thread for up to 10 seconds. Every other IPC handler trying to
//! `state.lock().await` (including the 500 ms stats push) queues up behind it.
//! After enough starvation cycles the WebView receives no JS callbacks and renders
//! a solid grey screen.
//!
//! Fix: removed `block_in_place`. The SS listener is spawned with a brief
//! `spawn_blocking` delay that does not hold the AppState lock (the lock is
//! released before the proxy pool is built). The first few requests may get
//! ECONNREFUSED if the listener hasn't bound yet, but the worker already retries
//! on connection errors, so no credentials are lost.
//!
//! **Root cause 2 — stale listener after SS server process dies:**
//! The tokio task running `server.run()` exits silently on network errors or OS
//! resource pressure. The pool returned the cached `socks5://127.0.0.1:<port>` URL
//! forever, causing all subsequent requests to hit ECONNREFUSED. The UI appeared
//! frozen because every worker was spinning on retries against a dead port.
//!
//! Fix: added a liveness probe in `resolve()`. On the fast path (already in map),
//! we check whether the port is still accepting connections. If not, we remove the
//! stale entry and re-spawn a fresh listener before returning.
//!
//! ## Fix for issue #44 — black screen with SS proxy (v0.4.8 regression)
//!
//! **Root cause — blocking TCP connect inside mutex lock:**
//! The liveness probe `port_is_alive()` was called while holding the `std::sync::Mutex`
//! lock. With multiple SS proxies, each 100ms TCP connect blocked the entire tokio
//! runtime, starving the WebView and causing a black screen with high CPU.
//!
//! Fix: move the `port_is_alive()` call OUTSIDE the mutex lock scope. Clone the
//! cached value first, drop the lock, then probe liveness. Only re-acquire the lock
//! when we need to remove a dead entry.
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
    net::{SocketAddr, TcpListener, TcpStream},
    sync::Mutex,
    time::Duration,
};

use shadowsocks::{
    config::{Mode, ServerConfig},
    crypto::CipherKind,
    ServerAddr,
};
use shadowsocks_service::{
    config::{
        Config, ConfigType, LocalConfig, LocalInstanceConfig, ProtocolType, ServerInstanceConfig,
    },
    local::Server,
};
use tokio::{runtime::Handle, sync::oneshot, task::JoinHandle};

/// Global singleton pool — initialised once per process.
static POOL: std::sync::OnceLock<ShadowsocksPool> = std::sync::OnceLock::new();

enum SsTaskHandle {
    Tokio(JoinHandle<()>),
    ThreadRuntime(oneshot::Sender<()>),
}

impl SsTaskHandle {
    fn abort(self) {
        match self {
            SsTaskHandle::Tokio(handle) => handle.abort(),
            SsTaskHandle::ThreadRuntime(shutdown_tx) => {
                let _ = shutdown_tx.send(());
            }
        }
    }
}

struct SsEntry {
    local_url: String,
    task_handle: SsTaskHandle,
}

pub struct ShadowsocksPool {
    /// Maps canonical ss:// URL → (local URL, task handle).
    /// FIX: Track task handles so we can abort stale servers and prevent memory leaks.
    map: Mutex<HashMap<String, SsEntry>>,
}

impl ShadowsocksPool {
    fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }

    /// Get-or-create the local SOCKS5 proxy URL for a given `ss://` URI.
    /// Returns the `socks5://127.0.0.1:<port>` string on success.
    ///
    /// This function is intentionally synchronous and non-blocking — it must be
    /// safe to call from sync contexts (e.g. `build_proxy_pool`) that run while
    /// the AppState tokio Mutex is held. No `block_in_place` or thread-sleeping
    /// occurs here. The SS listener is spawned via `tokio::spawn` (fire-and-forget);
    /// the first few worker requests may get a transient ECONNREFUSED which the
    /// worker already retries on.
    ///
    /// FIX: Track task handles and abort stale servers to prevent memory leaks.
    pub fn resolve(&self, ss_url: &str) -> Result<String, String> {
        // Fast path — already started. Probe liveness OUTSIDE the lock to avoid blocking.
        let cached_url = {
            let guard = self.map.lock().unwrap();
            guard.get(ss_url).map(|entry| entry.local_url.clone())
        };

        if let Some(local) = cached_url {
            let port = extract_port(&local);
            // CRITICAL: port_is_alive() is a 100ms TCP connect — must be OUTSIDE the lock
            if port.map(|p| port_is_alive(p)).unwrap_or(false) {
                return Ok(local);
            }
            // Port is dead — abort the task and remove entry
            eprintln!("[ss-pool] listener for {ss_url} is dead, aborting and re-spawning");
            let mut guard = self.map.lock().unwrap();
            if let Some(entry) = guard.remove(ss_url) {
                entry.task_handle.abort();
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
        let svr_cfg = ServerConfig::new(ss_server_addr, parsed.password.clone(), parsed.method)
            .map_err(|e| format!("bad SS server config: {e}"))?;

        let mut config = Config::new(ConfigType::Local);
        config
            .server
            .push(ServerInstanceConfig::with_server_config(svr_cfg));

        let mut local_cfg = LocalConfig::new(ProtocolType::Socks);
        local_cfg.addr = Some(ServerAddr::SocketAddr(listen_addr));
        local_cfg.mode = Mode::TcpOnly;
        config
            .local
            .push(LocalInstanceConfig::with_local_config(local_cfg));

        // Spawn the local SS server on the current tokio runtime when available.
        // Some unit tests call this path outside any runtime, so fall back to a
        // dedicated background thread with its own runtime instead of panicking.
        // We still keep the call non-blocking for the UI path.
        let task_handle = spawn_ss_server(config, port);

        let mut guard = self.map.lock().unwrap();
        // Double-check in case another thread raced us
        if let Some(existing) = guard.get(ss_url) {
            // Abort the task we just spawned since we already have one
            task_handle.abort();
            return Ok(existing.local_url.clone());
        }
        guard.insert(
            ss_url.to_string(),
            SsEntry {
                local_url: local_url.clone(),
                task_handle,
            },
        );
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
    for (canonical, entry) in guard.iter() {
        if entry.local_url == local_url {
            return Some(canonical.clone());
        }
    }
    None
}

/// Cleanup function to abort all tracked Shadowsocks server tasks.
/// Call this on application shutdown to prevent lingering background tasks.
pub fn shutdown_all() {
    let mut guard = pool().map.lock().unwrap();
    let count = guard.len();
    if count > 0 {
        eprintln!("[ss-pool] Shutting down {} shadowsocks servers", count);
        for (ss_url, entry) in guard.drain() {
            eprintln!("[ss-pool] Aborting task for {}", ss_url);
            entry.task_handle.abort();
        }
    }
}

fn spawn_ss_server(config: Config, port: u16) -> SsTaskHandle {
    let run_server = async move {
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
        eprintln!("[ss-pool] task for port {} terminated", port);
    };

    if Handle::try_current().is_ok() {
        return SsTaskHandle::Tokio(tokio::spawn(run_server));
    }

    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    std::thread::spawn(move || {
        match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => {
                rt.block_on(async move {
                    tokio::select! {
                        _ = run_server => {}
                        _ = async {
                            let _ = shutdown_rx.await;
                        } => {
                            eprintln!("[ss-pool] shutdown requested for port {}", port);
                        }
                    }
                });
            }
            Err(e) => {
                eprintln!("[ss-pool] failed to create fallback runtime for port {port}: {e}");
            }
        }
    });

    SsTaskHandle::ThreadRuntime(shutdown_tx)
}

// ── URI parsing ─────────────────────────────────────────────────────────────

struct ParsedSsUri {
    method: CipherKind,
    password: String,
    host: String,
    port: u16,
}

/// Parse `ss://method:password@host:port` or `ss://BASE64@host:port` (SIP002).
fn parse_ss_uri(uri: &str) -> Result<ParsedSsUri, String> {
    // Strip the `ss://` prefix (caller should already have it)
    let rest = uri.strip_prefix("ss://").unwrap_or(uri);

    // Split on the last '@' to get userinfo@hostinfo
    let at = rest
        .rfind('@')
        .ok_or_else(|| format!("missing '@' in ss URI: {uri}"))?;
    let userinfo = &rest[..at];
    let hostinfo = &rest[at + 1..];

    // Parse host:port
    let (host, port) = if let Some(colon) = hostinfo.rfind(':') {
        let h = hostinfo[..colon].to_string();
        let p: u16 = hostinfo[colon + 1..]
            .parse()
            .map_err(|_| format!("invalid port in ss URI: {uri}"))?;
        (h, p)
    } else {
        return Err(format!("missing port in ss URI: {uri}"));
    };

    // Parse method:password (or BASE64-encoded method:password per SIP002)
    let (method_str, password) = if let Some(colon) = userinfo.find(':') {
        // Cleartext: method:password
        (
            userinfo[..colon].to_string(),
            userinfo[colon + 1..].to_string(),
        )
    } else {
        // Try BASE64 decode (SIP002: base64url(method:password))
        let decoded = base64_decode(userinfo)
            .ok_or_else(|| format!("cannot decode userinfo in ss URI: {uri}"))?;
        if let Some(colon) = decoded.find(':') {
            (
                decoded[..colon].to_string(),
                decoded[colon + 1..].to_string(),
            )
        } else {
            return Err(format!("decoded ss userinfo has no ':': {decoded}"));
        }
    };

    let method: CipherKind = method_str.parse()
        .map_err(|_| format!("unknown SS cipher '{}' — supported: aes-128-gcm, aes-256-gcm, chacha20-ietf-poly1305, aes-128-gcm-2022, ...", method_str))?;

    Ok(ParsedSsUri {
        method,
        password,
        host,
        port,
    })
}

fn base64_decode(s: &str) -> Option<String> {
    // Try standard and URL-safe base64
    let padded = {
        let mut p = s.to_string();
        while p.len() % 4 != 0 {
            p.push('=');
        }
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
        if val == 255 {
            return None;
        }
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

/// Extract the port number from a `socks5://127.0.0.1:<port>` URL.
fn extract_port(local_url: &str) -> Option<u16> {
    local_url.rsplit(':').next()?.parse().ok()
}

/// Non-blocking liveness check: attempt a TCP connection to 127.0.0.1:<port>.
/// Returns true if the port is accepting connections, false otherwise.
/// Uses a short 100 ms timeout so the caller is never stalled.
fn port_is_alive(port: u16) -> bool {
    let addr = format!("127.0.0.1:{}", port);
    TcpStream::connect_timeout(
        &addr.parse().expect("valid loopback addr"),
        Duration::from_millis(100),
    )
    .is_ok()
}
