//! MITM proxy for browser capture.
//!
//! - Generates an ephemeral CA on first call (cached in AppState).
//! - For HTTP: forwards and captures plaintext.
//! - For HTTPS (CONNECT): intercepts TLS, signs a per-host cert with the CA,
//!   decrypts the request, re-encrypts to the real upstream.
//! - Tags every request/response with a `session_id` derived from the TCP
//!   connection that issued the CONNECT — this is how we get "tab isolation":
//!   each Chrome tab opens its own CONNECT tunnel per origin.

use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;

// ── CA / cert generation ────────────────────────────────────────────────────

pub struct MitmCa {
    pub cert_pem: String,   // PEM of the CA cert (to install in Chrome / browser)
    ca_cert: rcgen::Certificate,
    ca_key:  rcgen::KeyPair,
}

impl MitmCa {
    pub fn generate() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let ca_key = rcgen::KeyPair::generate()?;
        let mut params = rcgen::CertificateParams::new(vec![])?;
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        params.distinguished_name.push(rcgen::DnType::CommonName, "IronBullet Inspector CA");
        params.distinguished_name.push(rcgen::DnType::OrganizationName, "IronBullet");
        params.key_usages = vec![
            rcgen::KeyUsagePurpose::KeyCertSign,
            rcgen::KeyUsagePurpose::CrlSign,
        ];
        let ca_cert = params.self_signed(&ca_key)?;
        let cert_pem = ca_cert.pem();
        Ok(MitmCa { cert_pem, ca_cert, ca_key })
    }

    /// Sign a leaf certificate for `hostname`.
    fn sign_for_host(&self, hostname: &str) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
        let leaf_key = rcgen::KeyPair::generate()?;
        let mut params = rcgen::CertificateParams::new(vec![hostname.to_string()])?;
        params.distinguished_name.push(rcgen::DnType::CommonName, hostname);
        // signed_by signs the leaf cert with the CA cert + key directly — no re-parsing needed
        let leaf_cert = params.signed_by(&leaf_key, &self.ca_cert, &self.ca_key)?;
        Ok((leaf_cert.pem(), leaf_key.serialize_pem()))
    }
}

// ── Event emit helper ────────────────────────────────────────────────────────

fn emit(tx: &Sender<String>, payload: serde_json::Value) {
    use crate::ipc::IpcResponse;
    let resp = IpcResponse::ok("inspector_proxy_event", payload);
    let _ = tx.try_send(format!("window.__ipc_callback({})",
        serde_json::to_string(&resp).unwrap_or_default()));
}

// ── Connection handler ───────────────────────────────────────────────────────

pub async fn handle_connection(
    mut client: TcpStream,
    ca: Arc<MitmCa>,
    js: Sender<String>,
    session_id: String,
) {
    let mut buf = Vec::with_capacity(8192);
    // Read until \r\n\r\n
    loop {
        let mut b = [0u8; 1];
        match client.read(&mut b).await {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
        buf.push(b[0]);
        if buf.len() >= 4 && &buf[buf.len()-4..] == b"\r\n\r\n" { break; }
        if buf.len() > 65536 { return; }
    }

    let header_str = String::from_utf8_lossy(&buf).to_string();
    let first_line = header_str.lines().next().unwrap_or("").to_string();
    let parts: Vec<&str> = first_line.splitn(3, ' ').collect();
    if parts.len() < 2 { return; }

    let method = parts[0].to_string();
    let target = parts[1].to_string();

    if method == "CONNECT" {
        handle_connect(client, ca, js, session_id, target).await;
    } else {
        handle_http(client, buf, method, target, js, session_id, false).await;
    }
}

// ── HTTPS CONNECT + MITM ─────────────────────────────────────────────────────

async fn handle_connect(
    mut client: TcpStream,
    ca: Arc<MitmCa>,
    js: Sender<String>,
    session_id: String,
    target: String,
) {
    // Acknowledge CONNECT
    if client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await.is_err() { return; }

    // Parse host:port
    let (hostname, port) = split_host_port(&target, 443);

    // Sign a cert for this hostname
    let (cert_pem, key_pem) = match ca.sign_for_host(&hostname) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("[mitm] cert sign failed for {hostname}: {e}");
            return;
        }
    };

    // Wrap client stream in TLS (server role)
    let tls_acceptor = match make_tls_acceptor(&cert_pem, &key_pem) {
        Ok(a) => a,
        Err(e) => { eprintln!("[mitm] TLS acceptor error: {e}"); return; }
    };
    let tls_client = match tls_acceptor.accept(client).await {
        Ok(s) => s,
        Err(e) => { eprintln!("[mitm] TLS accept error for {hostname}: {e}"); return; }
    };
    let mut tls_client = tokio::io::BufStream::new(tls_client);

    // Loop: read HTTP requests from the now-decrypted stream, proxy to upstream
    loop {
        let mut req_buf = Vec::with_capacity(8192);
        // Read until \r\n\r\n
        let mut got_header = false;
        loop {
            let mut b = [0u8; 1];
            match tls_client.read(&mut b).await {
                Ok(0) | Err(_) => return, // connection closed
                Ok(_) => {}
            }
            req_buf.push(b[0]);
            if req_buf.len() >= 4 && &req_buf[req_buf.len()-4..] == b"\r\n\r\n" {
                got_header = true;
                break;
            }
            if req_buf.len() > 65536 { return; }
        }
        if !got_header { return; }

        let req_str = String::from_utf8_lossy(&req_buf).to_string();
        let first = req_str.lines().next().unwrap_or("").to_string();
        let parts: Vec<&str> = first.splitn(3, ' ').collect();
        if parts.len() < 2 { return; }
        let method = parts[0].to_string();
        let path   = parts[1].to_string();
        let url    = format!("https://{}:{}{}", hostname, port, path);

        // Parse headers + read body
        let (req_headers, content_length, is_chunked) = parse_headers(&req_str);
        let req_body = read_body(&mut tls_client, content_length, is_chunked).await;
        let body_str = String::from_utf8_lossy(&req_body).to_string();

        let req_id = uuid::Uuid::new_v4().to_string();
        emit(&js, serde_json::json!({
            "type":        "request",
            "id":          req_id,
            "session_id":  session_id,
            "method":      method,
            "url":         url,
            "host":        format!("{}:{}", hostname, port),
            "resource_type": infer_resource_type(&url, &req_headers),
            "headers":     req_headers,
            "post_data":   if body_str.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(body_str) }
        }));

        // Forward to real upstream over TLS
        let upstream_addr = format!("{}:{}", hostname, port);
        let upstream_tcp = match TcpStream::connect(&upstream_addr).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[mitm] upstream connect failed {upstream_addr}: {e}");
                let err = format!("HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n");
                let _ = tls_client.write_all(err.as_bytes()).await;
                return;
            }
        };

        let tls_connector = make_tls_connector(&hostname);
        let domain = rustls::pki_types::ServerName::try_from(hostname.clone())
            .unwrap_or_else(|_| rustls::pki_types::ServerName::try_from("localhost").unwrap());
        let mut upstream_tls = match tls_connector.connect(domain, upstream_tcp).await {
            Ok(s) => tokio::io::BufStream::new(s),
            Err(e) => {
                eprintln!("[mitm] upstream TLS failed {hostname}: {e}");
                return;
            }
        };

        // Write request to upstream
        upstream_tls.write_all(&req_buf).await.ok();
        if !req_body.is_empty() { upstream_tls.write_all(&req_body).await.ok(); }
        upstream_tls.flush().await.ok();

        // Read upstream response header
        let mut resp_buf = Vec::with_capacity(8192);
        loop {
            let mut b = [0u8; 1];
            match upstream_tls.read(&mut b).await {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
            resp_buf.push(b[0]);
            if resp_buf.len() >= 4 && &resp_buf[resp_buf.len()-4..] == b"\r\n\r\n" { break; }
            if resp_buf.len() > 65536 { break; }
        }
        if resp_buf.is_empty() { return; }

        let resp_str = String::from_utf8_lossy(&resp_buf).to_string();
        let (resp_status, resp_status_text) = parse_status_line(&resp_str);
        let (resp_headers, resp_cl, resp_chunked) = parse_headers(&resp_str);
        let resp_body = read_body(&mut upstream_tls, resp_cl, resp_chunked).await;

        // Relay response back to Chrome
        tls_client.write_all(&resp_buf).await.ok();
        tls_client.write_all(&resp_body).await.ok();
        tls_client.flush().await.ok();

        let resp_body_str = String::from_utf8_lossy(&resp_body);
        let mime = resp_headers.get("content-type").cloned().unwrap_or_default();
        emit(&js, serde_json::json!({
            "type":            "response",
            "id":              req_id,
            "session_id":      session_id,
            "resp_status":     resp_status,
            "resp_status_text": resp_status_text,
            "resp_mime":       mime,
            "resp_headers":    resp_headers,
            "resp_body":       resp_body_str.chars().take(131072).collect::<String>()
        }));

        // Check if we should close (Connection: close or HTTP/1.0)
        let conn_hdr = resp_headers.get("connection").cloned().unwrap_or_default();
        if conn_hdr.to_lowercase().contains("close") || resp_str.starts_with("HTTP/1.0") {
            return;
        }
    }
}

// ── Plain HTTP ───────────────────────────────────────────────────────────────

async fn handle_http(
    mut client: TcpStream,
    req_buf: Vec<u8>,
    method: String,
    target: String,
    js: Sender<String>,
    session_id: String,
    _is_https: bool,
) {
    let req_str = String::from_utf8_lossy(&req_buf).to_string();
    let (headers, content_length, is_chunked) = parse_headers(&req_str);

    let host_hdr = headers.get("host").cloned().unwrap_or_default();
    let url = if target.starts_with("http") { target.clone() }
              else { format!("http://{}{}", host_hdr, target) };

    let mut body_buf = Vec::new();
    if content_length > 0 {
        body_buf.resize(content_length.min(1024 * 1024), 0);
        let _ = client.read_exact(&mut body_buf).await;
    }
    let body_str = String::from_utf8_lossy(&body_buf).to_string();

    let req_id = uuid::Uuid::new_v4().to_string();
    emit(&js, serde_json::json!({
        "type":        "request",
        "id":          req_id,
        "session_id":  session_id,
        "method":      method,
        "url":         url,
        "host":        host_hdr,
        "resource_type": infer_resource_type(&url, &headers),
        "headers":     headers,
        "post_data":   if body_str.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(body_str) }
    }));

    let (upstream_host, upstream_port) = split_host_port(&host_hdr, 80);
    let upstream_addr = format!("{}:{}", upstream_host, upstream_port);
    let mut upstream = match TcpStream::connect(&upstream_addr).await {
        Ok(s) => s,
        Err(e) => {
            let _ = client.write_all(format!("HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n{e}").as_bytes()).await;
            return;
        }
    };

    upstream.write_all(&req_buf).await.ok();
    if !body_buf.is_empty() { upstream.write_all(&body_buf).await.ok(); }

    let mut resp_buf = Vec::with_capacity(8192);
    loop {
        let mut b = [0u8; 1];
        match upstream.read(&mut b).await {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
        resp_buf.push(b[0]);
        if resp_buf.len() >= 4 && &resp_buf[resp_buf.len()-4..] == b"\r\n\r\n" { break; }
        if resp_buf.len() > 65536 { break; }
    }

    let resp_str = String::from_utf8_lossy(&resp_buf).to_string();
    let (resp_status, resp_status_text) = parse_status_line(&resp_str);
    let (resp_headers, resp_cl, resp_chunked) = parse_headers(&resp_str);

    let mut resp_body = Vec::new();
    if resp_cl > 0 {
        resp_body.resize(resp_cl.min(2 * 1024 * 1024), 0);
        let _ = upstream.read_exact(&mut resp_body).await;
    } else if resp_chunked {
        let mut tmp = vec![0u8; 65536];
        loop {
            let n = upstream.read(&mut tmp).await.unwrap_or(0);
            if n == 0 { break; }
            resp_body.extend_from_slice(&tmp[..n]);
            if resp_body.len() > 2 * 1024 * 1024 { break; }
        }
    }

    client.write_all(&resp_buf).await.ok();
    client.write_all(&resp_body).await.ok();

    let resp_body_str = String::from_utf8_lossy(&resp_body);
    let mime = resp_headers.get("content-type").cloned().unwrap_or_default();
    emit(&js, serde_json::json!({
        "type":            "response",
        "id":              req_id,
        "session_id":      session_id,
        "resp_status":     resp_status,
        "resp_status_text": resp_status_text,
        "resp_mime":       mime,
        "resp_headers":    resp_headers,
        "resp_body":       resp_body_str.chars().take(131072).collect::<String>()
    }));
}

// ── TLS helpers ───────────────────────────────────────────────────────────────

fn make_tls_acceptor(cert_pem: &str, key_pem: &str)
    -> Result<tokio_rustls::TlsAcceptor, Box<dyn std::error::Error + Send + Sync>>
{
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};
    use rustls::ServerConfig;

    let cert_der = rustls_pemfile::certs(&mut cert_pem.as_bytes())
        .collect::<Result<Vec<CertificateDer<'static>>, _>>()?;
    let key_der  = rustls_pemfile::private_key(&mut key_pem.as_bytes())?
        .ok_or("no private key")?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_der, PrivateKeyDer::from(key_der))?;
    Ok(tokio_rustls::TlsAcceptor::from(Arc::new(config)))
}

fn make_tls_connector(hostname: &str) -> tokio_rustls::TlsConnector {
    use rustls::ClientConfig;
    use rustls::RootCertStore;

    let mut root_store = RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    // If hostname is an IP or known-invalid hostname, still try
    let _ = hostname; // used by caller for ServerName
    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    tokio_rustls::TlsConnector::from(Arc::new(config))
}

// ── Parsing helpers ──────────────────────────────────────────────────────────

fn parse_headers(raw: &str) -> (std::collections::HashMap<String, String>, usize, bool) {
    let mut headers = std::collections::HashMap::new();
    let mut cl = 0usize;
    let mut chunked = false;
    for line in raw.lines().skip(1) {
        if line.is_empty() { break; }
        if let Some(colon) = line.find(':') {
            let k = line[..colon].trim().to_lowercase();
            let v = line[colon+1..].trim().to_string();
            if k == "content-length" { cl = v.parse().unwrap_or(0); }
            if k == "transfer-encoding" && v.contains("chunked") { chunked = true; }
            headers.insert(k, v);
        }
    }
    (headers, cl, chunked)
}

fn parse_status_line(raw: &str) -> (u16, String) {
    let first = raw.lines().next().unwrap_or("");
    let parts: Vec<&str> = first.splitn(3, ' ').collect();
    let status: u16 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let text = parts.get(2).unwrap_or(&"").to_string();
    (status, text)
}

fn split_host_port(hostport: &str, default_port: u16) -> (String, u16) {
    // Handle IPv6 [::1]:port
    if hostport.starts_with('[') {
        if let Some(end) = hostport.find(']') {
            let host = hostport[1..end].to_string();
            let port = hostport.get(end+2..).and_then(|p| p.parse().ok()).unwrap_or(default_port);
            return (host, port);
        }
    }
    if let Some(pos) = hostport.rfind(':') {
        let port: u16 = hostport[pos+1..].parse().unwrap_or(default_port);
        return (hostport[..pos].to_string(), port);
    }
    (hostport.to_string(), default_port)
}

async fn read_body<R: AsyncReadExt + Unpin>(
    stream: &mut R,
    content_length: usize,
    chunked: bool,
) -> Vec<u8> {
    const MAX: usize = 4 * 1024 * 1024; // 4MB cap
    if content_length > 0 {
        let to_read = content_length.min(MAX);
        let mut buf = vec![0u8; to_read];
        let _ = stream.read_exact(&mut buf).await;
        return buf;
    }
    if chunked {
        let mut out = Vec::new();
        let mut tmp = vec![0u8; 65536];
        loop {
            let n = stream.read(&mut tmp).await.unwrap_or(0);
            if n == 0 { break; }
            out.extend_from_slice(&tmp[..n]);
            if out.len() >= MAX { break; }
        }
        return out;
    }
    Vec::new()
}

fn infer_resource_type(url: &str, headers: &std::collections::HashMap<String, String>) -> &'static str {
    let accept = headers.get("accept").map(|s| s.as_str()).unwrap_or("");
    let content_type = headers.get("content-type").map(|s| s.as_str()).unwrap_or("");
    if content_type.contains("application/json") || accept.contains("application/json") {
        return "fetch";
    }
    if url.ends_with(".js") || url.contains(".js?") { return "script"; }
    if url.ends_with(".css") || url.contains(".css?") { return "stylesheet"; }
    if url.ends_with(".png") || url.ends_with(".jpg") || url.ends_with(".gif") || url.ends_with(".webp") || url.ends_with(".ico") {
        return "image";
    }
    if url.ends_with(".woff") || url.ends_with(".woff2") || url.ends_with(".ttf") { return "font"; }
    let path = url.split('?').next().unwrap_or(url);
    if path.ends_with('/') || !path.contains('.') { return "document"; }
    "other"
}
