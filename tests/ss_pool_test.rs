/// Integration test for the embedded Shadowsocks pool.
/// Uses PIA's public free shadowsocks servers to verify the tunnel works.
/// Run with: cargo test --test ss_pool_test -- --nocapture
use std::time::Duration;
use ironbullet::sidecar::shadowsocks_pool::resolve_ss_proxy;

#[tokio::test]
async fn test_ss_pool_pia_swiss() {
    let ss_url = "ss://aes-128-gcm:shadowsocks@158.173.152.166:443";
    println!("Resolving: {ss_url}");

    let local_proxy = resolve_ss_proxy(ss_url);
    println!("Local tunnel: {local_proxy}");
    assert!(local_proxy.starts_with("socks5://127.0.0.1:"), "expected local socks5 URL, got: {local_proxy}");

    // Give the embedded server a moment to bind and be ready
    tokio::time::sleep(Duration::from_millis(1000)).await;

    let proxy = reqwest::Proxy::all(&local_proxy).expect("valid proxy URL");
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .proxy(proxy)
        .timeout(Duration::from_secs(20))
        .danger_accept_invalid_certs(true)
        .build()
        .expect("client build");

    let resp = client
        .get("https://ifconfig.co/json")
        .send()
        .await
        .expect("request should succeed through SS tunnel");

    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap_or_default();
    println!("Status: {status}\nBody: {body}");

    assert_eq!(status, 200, "expected 200 OK");
    assert!(
        body.contains("158.173.152.166"),
        "expected PIA Swiss IP (158.173.152.166) in response, got: {body}"
    );
    println!("✓ SS tunnel working — IP matches PIA Swiss server");
}

#[tokio::test]
async fn test_ss_uri_parse_cleartext() {
    // Just tests the resolve path doesn't panic; pool may or may not start
    let result = resolve_ss_proxy("ss://aes-128-gcm:shadowsocks@127.0.0.1:9999");
    // Should return a socks5:// URL even if the server isn't reachable
    println!("Parsed: {result}");
    assert!(result.starts_with("socks5://127.0.0.1:") || result.starts_with("ss://"));
}

#[tokio::test]
async fn test_ss_uri_parse_base64_sip002() {
    // SIP002: ss://BASE64(method:password)@host:port
    // aes-128-gcm:shadowsocks → base64 → "YWVzLTEyOC1nY206c2hhZG93c29ja3M="
    let b64 = base64_encode("aes-128-gcm:shadowsocks");
    let ss_url = format!("ss://{b64}@158.173.152.166:443");
    println!("SIP002 URL: {ss_url}");
    let result = resolve_ss_proxy(&ss_url);
    println!("Resolved: {result}");
    assert!(result.starts_with("socks5://127.0.0.1:") || result.starts_with("ss://"));
}

fn base64_encode(s: &str) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let bytes = s.as_bytes();
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let b0 = bytes[i] as u32;
        let b1 = if i+1 < bytes.len() { bytes[i+1] as u32 } else { 0 };
        let b2 = if i+2 < bytes.len() { bytes[i+2] as u32 } else { 0 };
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(if i+1 < bytes.len() { TABLE[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if i+2 < bytes.len() { TABLE[(n & 63) as usize] as char } else { '=' });
        i += 3;
    }
    out
}

#[tokio::test]
async fn test_canonical_reverse_lookup() {
    use ironbullet::sidecar::shadowsocks_pool::canonical_for_local;

    let ss_url = "ss://aes-128-gcm:shadowsocks@37.19.198.244:443"; // NJ server
    let local = resolve_ss_proxy(ss_url);
    println!("local: {local}");
    assert!(local.starts_with("socks5://127.0.0.1:"));

    // Reverse lookup should find the canonical ss:// URL
    let canonical = canonical_for_local(&local);
    println!("canonical: {canonical:?}");
    assert_eq!(canonical.as_deref(), Some(ss_url));

    // Unknown local URL returns None
    assert!(canonical_for_local("socks5://127.0.0.1:1").is_none());
}
