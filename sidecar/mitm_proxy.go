package main

import (
	"bufio"
	"bytes"
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/tls"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/json"
	"encoding/pem"
	"fmt"
	"io"
	"math/big"
	"net"
	"net/http"
	"net/url"
	"strings"
	"sync"
	"time"
)

// ── CA ───────────────────────────────────────────────────────────────────────

type MitmCA struct {
	cert    *x509.Certificate
	key     *ecdsa.PrivateKey
	certPEM string
	mu      sync.Mutex
	cache   map[string]*tls.Certificate // hostname → signed leaf cert
}

func NewMitmCA() (*MitmCA, error) {
	key, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		return nil, err
	}
	tmpl := &x509.Certificate{
		SerialNumber: big.NewInt(1),
		Subject:      pkix.Name{CommonName: "IronBullet Inspector CA", Organization: []string{"IronBullet"}},
		NotBefore:    time.Now().Add(-1 * time.Hour),
		NotAfter:     time.Now().Add(24 * time.Hour),
		IsCA:         true,
		KeyUsage:     x509.KeyUsageCertSign | x509.KeyUsageCRLSign,
		BasicConstraintsValid: true,
	}
	certDER, err := x509.CreateCertificate(rand.Reader, tmpl, tmpl, &key.PublicKey, key)
	if err != nil {
		return nil, err
	}
	cert, err := x509.ParseCertificate(certDER)
	if err != nil {
		return nil, err
	}
	certPEM := string(pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: certDER}))
	return &MitmCA{cert: cert, key: key, certPEM: certPEM, cache: make(map[string]*tls.Certificate)}, nil
}

func (ca *MitmCA) leafCert(hostname string) (*tls.Certificate, error) {
	ca.mu.Lock()
	if c, ok := ca.cache[hostname]; ok {
		ca.mu.Unlock()
		return c, nil
	}
	ca.mu.Unlock()

	key, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	if err != nil {
		return nil, err
	}
	tmpl := &x509.Certificate{
		SerialNumber: big.NewInt(time.Now().UnixNano()),
		Subject:      pkix.Name{CommonName: hostname},
		DNSNames:     []string{hostname},
		NotBefore:    time.Now().Add(-1 * time.Hour),
		NotAfter:     time.Now().Add(24 * time.Hour),
		KeyUsage:     x509.KeyUsageDigitalSignature,
		ExtKeyUsage:  []x509.ExtKeyUsage{x509.ExtKeyUsageServerAuth},
	}
	certDER, err := x509.CreateCertificate(rand.Reader, tmpl, ca.cert, &key.PublicKey, ca.key)
	if err != nil {
		return nil, err
	}
	tlsCert := &tls.Certificate{
		Certificate: [][]byte{certDER},
		PrivateKey:  key,
	}
	ca.mu.Lock()
	ca.cache[hostname] = tlsCert
	ca.mu.Unlock()
	return tlsCert, nil
}

// ── Proxy event ───────────────────────────────────────────────────────────────

type ProxyEvent struct {
	Type           string            `json:"type"`
	ID             string            `json:"id"`
	SessionID      string            `json:"session_id"`
	Method         string            `json:"method,omitempty"`
	URL            string            `json:"url,omitempty"`
	Host           string            `json:"host,omitempty"`
	ResourceType   string            `json:"resource_type,omitempty"`
	Headers        map[string]string `json:"headers,omitempty"`
	PostData       *string           `json:"post_data,omitempty"`
	RespStatus     int               `json:"resp_status,omitempty"`
	RespStatusText string            `json:"resp_status_text,omitempty"`
	RespMime       string            `json:"resp_mime,omitempty"`
	RespHeaders    map[string]string `json:"resp_headers,omitempty"`
	RespBody       string            `json:"resp_body,omitempty"`
	Message        string            `json:"message,omitempty"`
	Port           int               `json:"port,omitempty"`
	CACertPEM      string            `json:"ca_cert_pem,omitempty"`
}

// ── Proxy server ─────────────────────────────────────────────────────────────

type MitmProxy struct {
	ca        *MitmCA
	port      int
	listener  net.Listener
	events    chan ProxyEvent
	sessionID string // per-connection session tag (set per-accept)
}

func NewMitmProxy(port int) (*MitmProxy, error) {
	ca, err := NewMitmCA()
	if err != nil {
		return nil, fmt.Errorf("CA generation failed: %w", err)
	}
	ln, err := net.Listen("tcp", fmt.Sprintf("127.0.0.1:%d", port))
	if err != nil {
		return nil, fmt.Errorf("bind failed on port %d: %w", port, err)
	}
	actualPort := ln.Addr().(*net.TCPAddr).Port
	return &MitmProxy{
		ca:       ca,
		port:     actualPort,
		listener: ln,
		events:   make(chan ProxyEvent, 4096),
	}, nil
}

func (p *MitmProxy) CACertPEM() string { return p.ca.certPEM }
func (p *MitmProxy) Port() int         { return p.port }
func (p *MitmProxy) Stop()             { p.listener.Close() }

func (p *MitmProxy) Serve() {
	for {
		conn, err := p.listener.Accept()
		if err != nil {
			return // listener closed
		}
		sessionID := newUUID()
		go p.handleConn(conn, sessionID)
	}
}

func (p *MitmProxy) handleConn(conn net.Conn, sessionID string) {
	defer conn.Close()
	br := bufio.NewReader(conn)
	req, err := http.ReadRequest(br)
	if err != nil {
		return
	}

	if req.Method == "CONNECT" {
		p.handleConnect(conn, br, req, sessionID)
	} else {
		p.handleHTTP(conn, req, sessionID, false)
	}
}

func (p *MitmProxy) handleConnect(conn net.Conn, _ *bufio.Reader, req *http.Request, sessionID string) {
	host, _, err := net.SplitHostPort(req.Host)
	if err != nil {
		host = req.Host
	}

	// Acknowledge CONNECT
	conn.Write([]byte("HTTP/1.1 200 Connection Established\r\n\r\n"))

	// Get leaf cert for this host
	leafCert, err := p.ca.leafCert(host)
	if err != nil {
		return
	}

	// Wrap conn in TLS as server
	tlsConf := &tls.Config{
		Certificates: []tls.Certificate{*leafCert},
		MinVersion:   tls.VersionTLS12,
	}
	tlsConn := tls.Server(conn, tlsConf)
	if err := tlsConn.Handshake(); err != nil {
		return
	}
	defer tlsConn.Close()

	// Handle HTTP requests on the decrypted stream
	br := bufio.NewReader(tlsConn)
	for {
		req, err := http.ReadRequest(br)
		if err != nil {
			return
		}
		req.URL.Scheme = "https"
		req.URL.Host = req.Host
		if req.URL.Host == "" {
			req.URL.Host = host
		}
		if !p.forwardRequest(tlsConn, req, sessionID, true) {
			return
		}
	}
}

func (p *MitmProxy) handleHTTP(conn net.Conn, req *http.Request, sessionID string, _ bool) {
	p.forwardRequest(conn, req, sessionID, false)
}

// forwardRequest proxies one HTTP request, captures it, and writes the response back.
// Returns false if the connection should be closed.
func (p *MitmProxy) forwardRequest(conn net.Conn, req *http.Request, sessionID string, isTLS bool) bool {
	// Capture request
	reqID := newUUID()
	reqHeaders := headerMap(req.Header)
	var bodyBuf []byte
	if req.Body != nil {
		bodyBuf, _ = io.ReadAll(io.LimitReader(req.Body, 4*1024*1024))
		req.Body = io.NopCloser(bytes.NewReader(bodyBuf))
	}
	bodyStr := string(bodyBuf)

	fullURL := req.URL.String()
	ev := ProxyEvent{
		Type:         "request",
		ID:           reqID,
		SessionID:    sessionID,
		Method:       req.Method,
		URL:          fullURL,
		Host:         req.Host,
		ResourceType: inferResourceType(fullURL, req.Header),
		Headers:      reqHeaders,
	}
	if bodyStr != "" {
		ev.PostData = &bodyStr
	}
	p.emit(ev)

	// Build upstream request
	upstreamURL := *req.URL
	if upstreamURL.Scheme == "" {
		if isTLS {
			upstreamURL.Scheme = "https"
		} else {
			upstreamURL.Scheme = "http"
		}
	}
	if upstreamURL.Host == "" {
		upstreamURL.Host = req.Host
	}

	outReq, err := http.NewRequest(req.Method, upstreamURL.String(), bytes.NewReader(bodyBuf))
	if err != nil {
		conn.Write([]byte("HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n"))
		return false
	}
	// Copy headers, skip hop-by-hop
	for k, vv := range req.Header {
		if isHopByHop(k) {
			continue
		}
		for _, v := range vv {
			outReq.Header.Add(k, v)
		}
	}
	outReq.Header.Set("Host", req.Host)

	transport := &http.Transport{
		TLSClientConfig:    &tls.Config{InsecureSkipVerify: false},
		DisableCompression: true,
		ForceAttemptHTTP2:  false, // keep HTTP/1.1 for simplicity
	}
	client := &http.Client{
		Transport: transport,
		CheckRedirect: func(*http.Request, []*http.Request) error {
			return http.ErrUseLastResponse // don't follow redirects
		},
	}

	resp, err := client.Do(outReq)
	if err != nil {
		conn.Write([]byte("HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"))
		return false
	}
	defer resp.Body.Close()

	// Read response body
	respBody, _ := io.ReadAll(io.LimitReader(resp.Body, 4*1024*1024))

	// Write response back to Chrome
	var sb strings.Builder
	sb.WriteString(fmt.Sprintf("HTTP/1.1 %d %s\r\n", resp.StatusCode, resp.Status[strings.Index(resp.Status, " ")+1:]))
	for k, vv := range resp.Header {
		if isHopByHop(k) {
			continue
		}
		for _, v := range vv {
			sb.WriteString(fmt.Sprintf("%s: %s\r\n", k, v))
		}
	}
	sb.WriteString(fmt.Sprintf("Content-Length: %d\r\n", len(respBody)))
	// Always use Connection: keep-alive so Chrome reuses the tunnel
	sb.WriteString("Connection: keep-alive\r\n")
	sb.WriteString("\r\n")
	conn.Write([]byte(sb.String()))
	conn.Write(respBody)

	// Capture response
	respBodyStr := truncate(string(respBody), 131072)
	mime := resp.Header.Get("Content-Type")
	p.emit(ProxyEvent{
		Type:           "response",
		ID:             reqID,
		SessionID:      sessionID,
		RespStatus:     resp.StatusCode,
		RespStatusText: resp.Status[strings.Index(resp.Status, " ")+1:],
		RespMime:       mime,
		RespHeaders:    headerMap(resp.Header),
		RespBody:       respBodyStr,
	})

	return true
}

func (p *MitmProxy) emit(ev ProxyEvent) {
	select {
	case p.events <- ev:
	default:
	}
}

// ── Helpers ───────────────────────────────────────────────────────────────────

func headerMap(h http.Header) map[string]string {
	m := make(map[string]string, len(h))
	for k, vv := range h {
		m[strings.ToLower(k)] = strings.Join(vv, ", ")
	}
	return m
}

var hopByHopHeaders = map[string]bool{
	"Connection": true, "Proxy-Connection": true, "Keep-Alive": true,
	"Proxy-Authenticate": true, "Proxy-Authorization": true,
	"Te": true, "Trailers": true, "Transfer-Encoding": true, "Upgrade": true,
}

func isHopByHop(h string) bool { return hopByHopHeaders[http.CanonicalHeaderKey(h)] }

func inferResourceType(rawURL string, h http.Header) string {
	accept := h.Get("Accept")
	ct := h.Get("Content-Type")
	if strings.Contains(ct, "application/json") || strings.Contains(accept, "application/json") {
		return "fetch"
	}
	u, _ := url.Parse(rawURL)
	path := ""
	if u != nil {
		path = u.Path
	}
	switch {
	case strings.HasSuffix(path, ".js"):
		return "script"
	case strings.HasSuffix(path, ".css"):
		return "stylesheet"
	case strings.HasSuffix(path, ".png"), strings.HasSuffix(path, ".jpg"),
		strings.HasSuffix(path, ".gif"), strings.HasSuffix(path, ".webp"),
		strings.HasSuffix(path, ".ico"), strings.HasSuffix(path, ".svg"):
		return "image"
	case strings.HasSuffix(path, ".woff"), strings.HasSuffix(path, ".woff2"), strings.HasSuffix(path, ".ttf"):
		return "font"
	case strings.Contains(accept, "text/html"):
		return "document"
	}
	return "other"
}

func truncate(s string, max int) string {
	runes := []rune(s)
	if len(runes) > max {
		return string(runes[:max])
	}
	return s
}

func newUUID() string {
	b := make([]byte, 16)
	rand.Read(b)
	return fmt.Sprintf("%08x-%04x-%04x-%04x-%012x", b[0:4], b[4:6], b[6:8], b[8:10], b[10:])
}

// ── Sidecar integration ───────────────────────────────────────────────────────
// Handles action="start_mitm_proxy" and action="stop_mitm_proxy" from Rust.

var (
	activeMitmProxy *MitmProxy
	mitmProxyMu     sync.Mutex
)

func handleMitmProxyStart(req SidecarRequest) SidecarResponse {
	mitmProxyMu.Lock()
	defer mitmProxyMu.Unlock()

	// Stop any existing proxy
	if activeMitmProxy != nil {
		activeMitmProxy.Stop()
		activeMitmProxy = nil
	}

	port := 0 // auto-select
	if req.Body != "" {
		var cfg struct {
			Port int `json:"port"`
		}
		json.Unmarshal([]byte(req.Body), &cfg)
		port = cfg.Port
	}

	proxy, err := NewMitmProxy(port)
	if err != nil {
		return SidecarResponse{ID: req.ID, Error: err.Error()}
	}
	activeMitmProxy = proxy

	// Serve in background, stream events to stdout as JSON lines
	go proxy.Serve()
	go func() {
		for ev := range proxy.events {
			data, err := json.Marshal(ev)
			if err != nil {
				continue
			}
			// Write as a special "proxy_event" line that main.go can detect
			fmt.Printf("PROXY_EVENT:%s\n", string(data))
		}
	}()

	return SidecarResponse{
		ID:     req.ID,
		Status: 200,
		Body:   fmt.Sprintf(`{"port":%d,"ca_cert_pem":%s}`, proxy.Port(),
			jsonString(proxy.CACertPEM())),
	}
}

func handleMitmProxyStop(req SidecarRequest) SidecarResponse {
	mitmProxyMu.Lock()
	defer mitmProxyMu.Unlock()
	if activeMitmProxy != nil {
		activeMitmProxy.Stop()
		activeMitmProxy = nil
	}
	return SidecarResponse{ID: req.ID, Status: 200, Body: `{"stopped":true}`}
}

func jsonString(s string) string {
	b, _ := json.Marshal(s)
	return string(b)
}
