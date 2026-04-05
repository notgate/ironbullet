// Test suite for proxy parsing logic (Issue #56 fix verification)

use ironbullet::runner::proxy_pool::{ProxyPool, ProxyType};

#[test]
fn test_socks5_port_detection() {
    // Plain host:port on SOCKS5 port 1080 should be detected as SOCKS5
    let pool = ProxyPool::from_file_with_type("tests/fixtures/socks5_plain.txt", 300, None)
        .expect("Failed to load proxy file");

    let proxy = pool.next_proxy().expect("Should have a proxy");
    assert!(proxy.starts_with("socks5://"), "Port 1080 should be detected as SOCKS5, got: {}", proxy);
}

#[test]
fn test_explicit_scheme_takes_precedence() {
    // Explicitly prefixed http:// should remain HTTP even on port 1080
    let pool = ProxyPool::from_file_with_type("tests/fixtures/http_explicit.txt", 300, None)
        .expect("Failed to load proxy file");

    let proxy = pool.next_proxy().expect("Should have a proxy");
    assert!(proxy.starts_with("http://"), "Explicit http:// should be preserved, got: {}", proxy);
}

#[test]
fn test_tor_port_detection() {
    // Tor SOCKS5 port 9050 should be detected as SOCKS5
    let mut pool = ProxyPool::empty();
    pool.load_from_string("127.0.0.1:9050", None);

    let proxy = pool.next_proxy().expect("Should have a proxy");
    assert!(proxy.starts_with("socks5://"), "Port 9050 (Tor) should be detected as SOCKS5, got: {}", proxy);
}

#[test]
fn test_http_default_for_unknown_ports() {
    // Random ports should default to HTTP when no default_type is specified
    let mut pool = ProxyPool::empty();
    pool.load_from_string("192.168.1.1:8080", None);

    let proxy = pool.next_proxy().expect("Should have a proxy");
    assert!(proxy.starts_with("http://"), "Port 8080 should default to HTTP, got: {}", proxy);
}

#[test]
fn test_default_type_override() {
    // Explicit default_type should override port detection
    let mut pool = ProxyPool::empty();
    pool.load_from_string("192.168.1.1:8080", Some(ProxyType::Socks5));

    let proxy = pool.next_proxy().expect("Should have a proxy");
    assert!(proxy.starts_with("socks5://"), "default_type should override port detection, got: {}", proxy);
}

#[test]
fn test_authenticated_socks5() {
    // user:pass@host:port on SOCKS5 port should be detected correctly
    let mut pool = ProxyPool::empty();
    pool.load_from_string("testuser:testpass@127.0.0.1:1080", None);

    let proxy = pool.next_proxy().expect("Should have a proxy");
    assert!(proxy.starts_with("socks5://"), "Authenticated SOCKS5 should be detected, got: {}", proxy);
    assert!(proxy.contains("testuser"), "Username should be preserved");
    assert!(proxy.contains("testpass"), "Password should be preserved");
}

#[test]
fn test_colon_separated_auth_format() {
    // host:port:user:pass format on SOCKS5 port
    let mut pool = ProxyPool::empty();
    pool.load_from_string("127.0.0.1:1080:myuser:mypass", None);

    let proxy = pool.next_proxy().expect("Should have a proxy");
    assert!(proxy.starts_with("socks5://"), "Colon format SOCKS5 should be detected, got: {}", proxy);
    assert!(proxy.contains("myuser"), "Username should be preserved");
    assert!(proxy.contains("mypass"), "Password should be preserved");
}

#[test]
fn test_shadowsocks_detection() {
    // ss:// prefix should be preserved
    let mut pool = ProxyPool::empty();
    pool.load_from_string("ss://aes-128-gcm:password@example.com:8388", None);

    let proxy = pool.next_proxy().expect("Should have a proxy");
    // Shadowsocks should be converted to local socks5:// tunnel
    assert!(proxy.starts_with("socks5://127.0.0.1:"),
        "Shadowsocks should resolve to local SOCKS5 tunnel, got: {}", proxy);
}

#[test]
fn test_multiple_proxy_types_mixed() {
    // Mix of different proxy types should all be parsed correctly
    let mut pool = ProxyPool::empty();
    pool.load_from_string("http://proxy1.com:8080", None);      // Explicit HTTP
    pool.load_from_string("127.0.0.1:1080", None);               // Auto-detect SOCKS5
    pool.load_from_string("socks5://proxy2.com:1081", None);    // Explicit SOCKS5
    pool.load_from_string("192.168.1.1:3128", None);             // Auto HTTP

    assert_eq!(pool.total(), 4, "Should have 4 proxies");

    // Get all proxies and verify types
    let mut proxies = vec![];
    for _ in 0..4 {
        proxies.push(pool.next_proxy().expect("Should have proxy"));
    }

    assert!(proxies.iter().any(|p| p.starts_with("http://proxy1.com")), "Should have explicit HTTP");
    assert!(proxies.iter().any(|p| p.starts_with("socks5://127.0.0.1:1080")), "Should have auto-detected SOCKS5");
    assert!(proxies.iter().any(|p| p.starts_with("socks5://proxy2.com:1081")), "Should have explicit SOCKS5");
    assert!(proxies.iter().any(|p| p.starts_with("http://192.168.1.1:3128")), "Should have auto HTTP");
}
