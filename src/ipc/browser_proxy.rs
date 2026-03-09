//! MITM proxy for browser capture.
//!
//! Uses a proper buffered reader for HTTP header parsing — no byte-at-a-time reads.
//! For HTTPS: intercepts CONNECT, does TLS MITM with per-host signed certs.
//! Each CONNECT tunnel gets a unique session_id for tab isolation.

use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc::Sender;

// ── CA / cert generation ────────────────────────────────────────────────────

pub struct MitmCa {
    pub cert_pem: String,
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

    fn sign_for_host(&self, hostname: &str) -> Result<(String, String), Box<dyn std::error::Error + Send + Sync>> {
        let leaf_key = rcgen::KeyPair::generate()?;
        let mut params = rcgen::CertificateParams::new(vec![hostname.to_string()])?;
        params.distinguished_name.push(rcgen::DnType::CommonName, hostname);
        let leaf_cert = params.signed_by(&leaf_key, &self.ca_cert, &self.ca_key)?;
        Ok((leaf_cert.pem(), leaf_key.serialize_pem()))
    }
}

// ── Emit helper ──────────────────────────────────────────────────────────────

fn emit(tx: &Sender<String>, payload: serde_json::Value) {
    use crate::ipc::IpcResponse;
    let resp = IpcResponse::ok("inspector_proxy_event", payload);
    let _ = tx.try_send(format!("window.__ipc_callback({})",
        serde_json::to_string(&resp).unwrap_or_default()));
}

// ── Entry point ──────────────────────────────────────────────────────────────

pub async fn handle_connection(
    client: TcpStream,
    ca: Arc<MitmCa>,
    js: Sender<String>,
    session_id: String,
) {
    // Split the stream so we can read (buffered) and write independently
    let (read_half, write_half) = tokio::io::split(client);
    let mut reader = BufReader::new(read_half);

    let (method, target, raw_headers, headers) = match read_request_head(&mut reader).await {
        Some(v) => v,
        None => return,
    };

    if method == "CONNECT" {
        // Reassemble the stream from the split halves
        let read_half = reader.into_inner();
        let client = read_half.unsplit(write_half);
        handle_connect(client, ca, js, session_id, target).await;
    } else {
        handle_http_plain(reader, write_half, method, target, raw_headers, headers, js, session_id).await;
    }
}

// ── CONNECT / HTTPS MITM ─────────────────────────────────────────────────────

async fn handle_connect(
    mut client: TcpStream,
    ca: Arc<MitmCa>,
    js: Sender<String>,
    session_id: String,
    target: String,
) {
    // Ack the tunnel
    if client.write_all(b"HTTP/1.1 200 Connection Established\r\n\r\n").await.is_err() { return; }

    let (hostname, port) = split_host_port(&target, 443);

    // Sign a leaf cert for this hostname
    let (cert_pem, key_pem) = match ca.sign_for_host(&hostname) {
        Ok(v) => v,
        Err(e) => { eprintln!("[mitm] cert error for {hostname}: {e}"); return; }
    };

    // Accept TLS from Chrome
    let acceptor = match make_tls_acceptor(&cert_pem, &key_pem) {
        Ok(a) => a,
        Err(e) => { eprintln!("[mitm] acceptor error: {e}"); return; }
    };
    let tls_stream = match acceptor.accept(client).await {
        Ok(s) => s,
        Err(e) => { eprintln!("[mitm] TLS accept error for {hostname}: {e}"); return; }
    };
    let mut client_reader = BufReader::new(tls_stream);

    // Loop: handle multiple HTTP requests on this keep-alive tunnel
    loop {
        let (method, path, raw_headers, headers) = match read_request_head(&mut client_reader).await {
            Some(v) => v,
            None => break, // connection closed
        };

        let url = format!("https://{}:{}{}", hostname, port, path);

        // Read request body
        let req_body = read_body_from_headers(&mut client_reader, &headers).await;
        let body_str = String::from_utf8_lossy(&req_body).to_string();

        let req_id = uuid::Uuid::new_v4().to_string();
        emit(&js, serde_json::json!({
            "type":          "request",
            "id":            req_id,
            "session_id":    session_id,
            "method":        method,
            "url":           url,
            "host":          format!("{}:{}", hostname, port),
            "resource_type": infer_resource_type(&url, &headers),
            "headers":       headers,
            "post_data":     if body_str.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(body_str) }
        }));

        // Connect to real upstream over TLS (fresh connection per request — simple and correct)
        let upstream_addr = format!("{}:{}", hostname, port);
        let upstream_tcp = match TcpStream::connect(&upstream_addr).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[mitm] upstream connect failed {upstream_addr}: {e}");
                // Send 502 back to Chrome so it doesn't just hang
                let client_writer = client_reader.get_mut();
                let _ = client_writer.write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").await;
                break;
            }
        };

        let connector = make_tls_connector();
        let server_name = match rustls::pki_types::ServerName::try_from(hostname.clone()) {
            Ok(n) => n,
            Err(_) => break,
        };
        let upstream_tls = match connector.connect(server_name, upstream_tcp).await {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[mitm] upstream TLS failed {hostname}: {e}");
                let client_writer = client_reader.get_mut();
                let _ = client_writer.write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").await;
                break;
            }
        };
        let mut upstream_rw = tokio::io::BufStream::new(upstream_tls);

        // Forward request to upstream
        upstream_rw.write_all(raw_headers.as_bytes()).await.ok();
        if !req_body.is_empty() { upstream_rw.write_all(&req_body).await.ok(); }
        upstream_rw.flush().await.ok();

        // Read upstream response headers
        let (status, status_text, resp_raw_headers, resp_headers) = match read_response_head(&mut upstream_rw).await {
            Some(v) => v,
            None => break,
        };

        // Read body — for responses without content-length or chunked encoding,
        // read until upstream closes (connection: close or EOF).
        let has_cl = resp_headers.contains_key("content-length");
        let is_chunked = resp_headers.get("transfer-encoding")
            .map(|v| v.contains("chunked")).unwrap_or(false);
        let resp_body = if has_cl || is_chunked {
            read_body_from_headers(&mut upstream_rw, &resp_headers).await
        } else {
            // Read until EOF (upstream will close after response on HTTP/1.0 or Connection: close)
            let mut buf = Vec::new();
            let _ = upstream_rw.read_to_end(&mut buf).await;
            buf
        };

        // Relay response back to Chrome — must flush TLS stream explicitly
        let client_writer = client_reader.get_mut();
        client_writer.write_all(resp_raw_headers.as_bytes()).await.ok();
        client_writer.write_all(&resp_body).await.ok();
        // Flush the TLS write buffer so Chrome actually receives the data
        use tokio::io::AsyncWriteExt as _;
        client_writer.flush().await.ok();

        let resp_body_str = String::from_utf8_lossy(&resp_body);
        let mime = resp_headers.get("content-type").cloned().unwrap_or_default();
        emit(&js, serde_json::json!({
            "type":             "response",
            "id":               req_id,
            "session_id":       session_id,
            "resp_status":      status,
            "resp_status_text": status_text,
            "resp_mime":        mime,
            "resp_headers":     resp_headers,
            "resp_body":        resp_body_str.chars().take(131072).collect::<String>()
        }));

        // If no content-length/chunked, we read until EOF so the upstream is done.
        // Either way, Chrome expects Connection: keep-alive handled — since we open
        // a fresh upstream connection per request, always continue the client loop.
        let conn = resp_headers.get("connection").cloned().unwrap_or_default();
        if conn.to_lowercase().contains("close") || (!has_cl && !is_chunked) { break; }
    }
}

// ── Plain HTTP ───────────────────────────────────────────────────────────────

async fn handle_http_plain(
    mut reader: BufReader<tokio::io::ReadHalf<TcpStream>>,
    mut writer: tokio::io::WriteHalf<TcpStream>,
    method: String,
    target: String,
    raw_headers: String,
    headers: std::collections::HashMap<String, String>,
    js: Sender<String>,
    session_id: String,
) {
    let host_hdr = headers.get("host").cloned().unwrap_or_default();
    let url = if target.starts_with("http") { target.clone() }
              else { format!("http://{}{}", host_hdr, target) };

    let req_body = read_body_from_headers(&mut reader, &headers).await;
    let body_str = String::from_utf8_lossy(&req_body).to_string();

    let req_id = uuid::Uuid::new_v4().to_string();
    emit(&js, serde_json::json!({
        "type":          "request",
        "id":            req_id,
        "session_id":    session_id,
        "method":        method,
        "url":           url,
        "host":          host_hdr,
        "resource_type": infer_resource_type(&url, &headers),
        "headers":       headers,
        "post_data":     if body_str.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(body_str) }
    }));

    let (upstream_host, upstream_port) = split_host_port(&host_hdr, 80);
    let mut upstream = match TcpStream::connect(format!("{}:{}", upstream_host, upstream_port)).await {
        Ok(s) => s,
        Err(e) => {
            let _ = writer.write_all(
                format!("HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n{e}").as_bytes()
            ).await;
            return;
        }
    };

    upstream.write_all(raw_headers.as_bytes()).await.ok();
    if !req_body.is_empty() { upstream.write_all(&req_body).await.ok(); }

    let mut upstream_reader = BufReader::new(upstream);
    let (status, status_text, resp_raw, resp_headers) = match read_response_head(&mut upstream_reader).await {
        Some(v) => v,
        None => return,
    };
    let has_cl = resp_headers.contains_key("content-length");
    let is_chunked = resp_headers.get("transfer-encoding")
        .map(|v| v.contains("chunked")).unwrap_or(false);
    let resp_body = if has_cl || is_chunked {
        read_body_from_headers(&mut upstream_reader, &resp_headers).await
    } else {
        let mut buf = Vec::new();
        let _ = upstream_reader.read_to_end(&mut buf).await;
        buf
    };

    writer.write_all(resp_raw.as_bytes()).await.ok();
    writer.write_all(&resp_body).await.ok();
    writer.flush().await.ok();

    let mime = resp_headers.get("content-type").cloned().unwrap_or_default();
    emit(&js, serde_json::json!({
        "type":             "response",
        "id":               req_id,
        "session_id":       session_id,
        "resp_status":      status,
        "resp_status_text": status_text,
        "resp_mime":        mime,
        "resp_headers":     resp_headers,
        "resp_body":        String::from_utf8_lossy(&resp_body).chars().take(131072).collect::<String>()
    }));
}

// ── HTTP parsing ─────────────────────────────────────────────────────────────

async fn read_request_head<R: AsyncBufReadExt + Unpin>(
    reader: &mut R,
) -> Option<(String, String, String, std::collections::HashMap<String, String>)> {
    let mut lines = Vec::new();
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) | Err(_) => return None,
            Ok(_) => {}
        }
        let trimmed = line.trim_end_matches(['\r', '\n']).to_string();
        if trimmed.is_empty() { break; }
        lines.push(trimmed);
        if lines.len() > 200 { return None; } // guard
    }
    if lines.is_empty() { return None; }

    let parts: Vec<&str> = lines[0].splitn(3, ' ').collect();
    if parts.len() < 2 { return None; }
    let method = parts[0].to_string();
    let target = parts[1].to_string();

    let mut headers = std::collections::HashMap::new();
    for line in &lines[1..] {
        if let Some(colon) = line.find(':') {
            let k = line[..colon].trim().to_lowercase();
            let v = line[colon+1..].trim().to_string();
            headers.insert(k, v);
        }
    }

    // Reconstruct raw header block (needed to forward to upstream)
    let raw = lines.join("\r\n") + "\r\n\r\n";
    Some((method, target, raw, headers))
}

async fn read_response_head<R: AsyncBufReadExt + Unpin>(
    reader: &mut R,
) -> Option<(u16, String, String, std::collections::HashMap<String, String>)> {
    let mut lines = Vec::new();
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) | Err(_) => return None,
            Ok(_) => {}
        }
        let trimmed = line.trim_end_matches(['\r', '\n']).to_string();
        if trimmed.is_empty() { break; }
        lines.push(trimmed);
        if lines.len() > 200 { return None; }
    }
    if lines.is_empty() { return None; }

    let parts: Vec<&str> = lines[0].splitn(3, ' ').collect();
    let status: u16 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let status_text = parts.get(2).unwrap_or(&"").to_string();

    let mut headers = std::collections::HashMap::new();
    for line in &lines[1..] {
        if let Some(colon) = line.find(':') {
            let k = line[..colon].trim().to_lowercase();
            let v = line[colon+1..].trim().to_string();
            headers.insert(k, v);
        }
    }

    let raw = lines.join("\r\n") + "\r\n\r\n";
    Some((status, status_text, raw, headers))
}

async fn read_body_from_headers<R: AsyncBufReadExt + AsyncReadExt + Unpin>(
    reader: &mut R,
    headers: &std::collections::HashMap<String, String>,
) -> Vec<u8> {
    const MAX: usize = 4 * 1024 * 1024;
    let cl: usize = headers.get("content-length").and_then(|v| v.parse().ok()).unwrap_or(0);
    let chunked = headers.get("transfer-encoding")
        .map(|v| v.contains("chunked")).unwrap_or(false);

    if cl > 0 {
        let to_read = cl.min(MAX);
        let mut buf = vec![0u8; to_read];
        let _ = reader.read_exact(&mut buf).await;
        return buf;
    }
    if chunked {
        return read_chunked(reader, MAX).await;
    }
    Vec::new()
}

async fn read_chunked<R: AsyncBufReadExt + AsyncReadExt + Unpin>(reader: &mut R, max: usize) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        // Read chunk size line
        let mut size_line = String::new();
        if reader.read_line(&mut size_line).await.unwrap_or(0) == 0 { break; }
        let size_str = size_line.trim().split(';').next().unwrap_or("0");
        let chunk_size = usize::from_str_radix(size_str, 16).unwrap_or(0);
        if chunk_size == 0 { 
            // Read trailing CRLF
            let mut trail = String::new();
            let _ = reader.read_line(&mut trail).await;
            break;
        }
        let to_read = chunk_size.min(max - out.len());
        let mut chunk = vec![0u8; to_read];
        let _ = reader.read_exact(&mut chunk).await;
        out.extend_from_slice(&chunk);
        // Skip leftover chunk bytes if we capped
        if chunk_size > to_read {
            let mut skip = vec![0u8; chunk_size - to_read];
            let _ = reader.read_exact(&mut skip).await;
        }
        // Read trailing CRLF after chunk data
        let mut crlf = String::new();
        let _ = reader.read_line(&mut crlf).await;
        if out.len() >= max { break; }
    }
    out
}

// ── TLS helpers ───────────────────────────────────────────────────────────────

fn make_tls_acceptor(cert_pem: &str, key_pem: &str)
    -> Result<tokio_rustls::TlsAcceptor, Box<dyn std::error::Error + Send + Sync>>
{
    use rustls::pki_types::{CertificateDer, PrivateKeyDer};
    use rustls::ServerConfig;

    let cert_der = rustls_pemfile::certs(&mut cert_pem.as_bytes())
        .collect::<Result<Vec<CertificateDer<'static>>, _>>()?;
    let key_der = rustls_pemfile::private_key(&mut key_pem.as_bytes())?
        .ok_or("no private key")?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_der, PrivateKeyDer::from(key_der))?;
    Ok(tokio_rustls::TlsAcceptor::from(Arc::new(config)))
}

fn make_tls_connector() -> tokio_rustls::TlsConnector {
    use rustls::{ClientConfig, RootCertStore};
    let mut root_store = RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    tokio_rustls::TlsConnector::from(Arc::new(config))
}

// ── Misc helpers ─────────────────────────────────────────────────────────────

fn split_host_port(hostport: &str, default_port: u16) -> (String, u16) {
    if hostport.starts_with('[') {
        if let Some(end) = hostport.find(']') {
            let host = hostport[1..end].to_string();
            let port = hostport.get(end+2..).and_then(|p| p.parse().ok()).unwrap_or(default_port);
            return (host, port);
        }
    }
    if let Some(pos) = hostport.rfind(':') {
        if let Ok(port) = hostport[pos+1..].parse::<u16>() {
            return (hostport[..pos].to_string(), port);
        }
    }
    (hostport.to_string(), default_port)
}

fn infer_resource_type(url: &str, headers: &std::collections::HashMap<String, String>) -> &'static str {
    let accept = headers.get("accept").map(|s| s.as_str()).unwrap_or("");
    let ct = headers.get("content-type").map(|s| s.as_str()).unwrap_or("");
    if ct.contains("application/json") || accept.contains("application/json") { return "fetch"; }
    let path = url.split('?').next().unwrap_or(url);
    if path.ends_with(".js")   || path.contains(".js?")  { return "script"; }
    if path.ends_with(".css")  || path.contains(".css?") { return "stylesheet"; }
    if path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".gif")
        || path.ends_with(".webp") || path.ends_with(".ico") || path.ends_with(".svg") { return "image"; }
    if path.ends_with(".woff") || path.ends_with(".woff2") || path.ends_with(".ttf") { return "font"; }
    if accept.contains("text/html") { return "document"; }
    "other"
}
