package main

import (
	"context"
	"fmt"
	"net"
	"net/url"
	"os"
	"strings"
	"sync"
	"time"

	"github.com/Noooste/azuretls-client"
)

type SessionWrapper struct {
	// mu guards all mutable fields below (Browser, CurrentProxy, LastJA3,
	// LastHTTP2FP, LastUsed). The map-level sessionsMu only protects the map
	// itself; two goroutines may hold the same *SessionWrapper simultaneously
	// (e.g. two HTTP blocks in the same pipeline run on different workers),
	// so each SessionWrapper needs its own lock for field-level safety.
	mu           sync.Mutex
	Session      *azuretls.Session
	Browser      string
	// CurrentProxy tracks the proxy currently configured on this session.
	// SetProxy is expensive (rebuilds transport state in azuretls), so we
	// skip the call when the proxy hasn't changed since the last request.
	CurrentProxy string
	// LastJA3 / LastHTTP2FP track the last applied fingerprint strings.
	// ApplyJa3 and ApplyHTTP2 rebuild internal TLS state and reset HTTP/2
	// connection pooling — calling them on every request when the value
	// hasn't changed destroys throughput. Skip re-application when unchanged.
	LastJA3     string
	LastHTTP2FP string
	// LastUsed tracks the last request time for GC eviction.
	LastUsed    time.Time
}

// connectTimeout is the hard limit for establishing a TCP connection to a proxy
// or target host. Kept intentionally short: if a proxy takes >10s to accept a
// connection it is unusable regardless of the request timeout.
const connectTimeout = 10 * time.Second

func handleNewSession(req SidecarRequest) {
	session := azuretls.NewSession()

	// Set a short connect timeout via ModifyDialer so the plain (non-proxy) dialer
	// doesn't block for the OS TCP timeout (~2 min) on dead hosts.
	// Also set SO_LINGER=0 (RST instead of FIN on close) to immediately release
	// the local port instead of leaving it in TIME_WAIT for ~4 minutes.
	// This is critical on Windows where the ephemeral port range (~16k ports) is
	// exhausted quickly at high thread counts, causing WSAEADDRINUSE errors.
	session.ModifyDialer = func(d *net.Dialer) error {
		d.Timeout = connectTimeout
		d.KeepAlive = 30 * time.Second
		d.Control = soLingerZero
		return nil
	}

	// Set browser profile
	browser := req.Browser
	if browser == "" {
		browser = "chrome"
	}
	session.Browser = browser

	// Set custom JA3 if provided
	if req.JA3 != "" {
		if err := session.ApplyJa3(req.JA3, browser); err != nil {
			sendError(req.ID, fmt.Sprintf("Failed to apply JA3: %v", err))
			return
		}
	}

	// Set HTTP/2 fingerprint if provided
	if req.HTTP2FP != "" {
		if err := session.ApplyHTTP2(req.HTTP2FP); err != nil {
			sendError(req.ID, fmt.Sprintf("Failed to apply HTTP2 fingerprint: %v", err))
			return
		}
	}

	// Set proxy if provided
	if req.Proxy != "" {
		if err := session.SetProxy(req.Proxy); err != nil {
			sendError(req.ID, fmt.Sprintf("Failed to set proxy: %v", err))
			return
		}
	}

	// Configure redirects
	if req.FollowRedirects != nil && !*req.FollowRedirects {
		session.MaxRedirects = 0
	} else if req.MaxRedirects > 0 {
		session.MaxRedirects = uint(req.MaxRedirects)
	}

	// TLS verification — ssl_verify: false skips cert checks (for testing / self-signed)
	if req.SslVerify != nil && !*req.SslVerify {
		session.InsecureSkipVerify = true
	}

	// Custom cipher suites — override the browser profile's default cipher list
	if req.CustomCiphers != "" {
		if err := applyCustomCiphers(session, browser, req.CustomCiphers); err != nil {
			// Log but don't fail — fall back to browser default
			fmt.Fprintf(os.Stderr, "[sidecar] custom_ciphers error: %v\n", err)
		}
	}

	// Pre-initialize the transport so the first Do() doesn't pay the lazy-init
	// cost inline. InitTransport is idempotent; calling it here moves the cost
	// to session setup time and keeps request latency predictable.
	if err := session.InitTransport(browser); err != nil {
		// Non-fatal — the transport will self-initialize on first request.
		fmt.Fprintf(os.Stderr, "[sidecar] InitTransport warning: %v\n", err)
	}

	sessionsMu.Lock()
	sessions[req.Session] = &SessionWrapper{
		Session:     session,
		Browser:     browser,
		LastJA3:     req.JA3,
		LastHTTP2FP: req.HTTP2FP,
		LastUsed:    time.Now(),
	}
	sessionsMu.Unlock()

	sendResponse(SidecarResponse{
		ID:     req.ID,
		Status: 200,
		Body:   "session created",
	})
}

func handleCloseSession(req SidecarRequest) {
	sessionsMu.Lock()
	if sw, ok := sessions[req.Session]; ok {
		sw.Session.Close()
		delete(sessions, req.Session)
	}
	sessionsMu.Unlock()

	sendResponse(SidecarResponse{
		ID:     req.ID,
		Status: 200,
		Body:   "session closed",
	})
}

func handleSetProxy(req SidecarRequest) {
	sessionsMu.RLock()
	sw, ok := sessions[req.Session]
	sessionsMu.RUnlock()

	if !ok {
		sendError(req.ID, "Session not found")
		return
	}

	if err := sw.Session.SetProxy(req.Proxy); err != nil {
		sendError(req.ID, fmt.Sprintf("Failed to set proxy: %v", err))
		return
	}

	sendResponse(SidecarResponse{
		ID:     req.ID,
		Status: 200,
		Body:   "proxy set",
	})
}

func handleSetBrowser(req SidecarRequest) {
	sessionsMu.RLock()
	sw, ok := sessions[req.Session]
	sessionsMu.RUnlock()

	if !ok {
		sendError(req.ID, "Session not found")
		return
	}

	sw.Session.Browser = req.Browser

	if req.JA3 != "" {
		sw.Session.ApplyJa3(req.JA3, req.Browser)
	}

	sendResponse(SidecarResponse{
		ID:     req.ID,
		Status: 200,
		Body:   "browser set",
	})
}

func handleHTTPRequest(req SidecarRequest) {
	sessionsMu.RLock()
	sw, ok := sessions[req.Session]
	sessionsMu.RUnlock()

	if !ok {
		// Auto-create session under write lock to prevent double-creation race.
		// Two goroutines arriving simultaneously for the same session ID would
		// both read !ok and both call handleNewSession, leaking a session.
		sessionsMu.Lock()
		sw, ok = sessions[req.Session]
		if !ok {
			session := azuretls.NewSession()
			browser := req.Browser
			if browser == "" {
				browser = "chrome"
			}
			session.Browser = browser
			if req.Proxy != "" {
				if err := session.SetProxy(req.Proxy); err != nil {
					fmt.Fprintf(os.Stderr, "[sidecar] auto-create SetProxy error: %v\n", err)
				}
			}
			if req.SslVerify != nil && !*req.SslVerify {
				session.InsecureSkipVerify = true
			}
			sw = &SessionWrapper{Session: session, Browser: browser, LastUsed: time.Now()}
			sessions[req.Session] = sw
		}
		sessionsMu.Unlock()
	}

	// All mutable field access on sw must be under sw.mu.
	// The map lock (sessionsMu) only protects the sessions map itself; two
	// goroutines can hold the same *SessionWrapper simultaneously when a
	// pipeline has multiple HTTP blocks running on different Tokio workers.
	sw.mu.Lock()
	sw.LastUsed = time.Now()

	// Apply a per-block browser_profile change only when it actually differs.
	if req.Browser != "" && req.Browser != sw.Browser {
		sw.Session.Browser = req.Browser
		if req.JA3 != "" && req.JA3 != sw.LastJA3 {
			sw.Session.ApplyJa3(req.JA3, req.Browser)
			sw.LastJA3 = req.JA3
		}
		if req.HTTP2FP != "" && req.HTTP2FP != sw.LastHTTP2FP {
			sw.Session.ApplyHTTP2(req.HTTP2FP)
			sw.LastHTTP2FP = req.HTTP2FP
		}
		sw.Browser = req.Browser
	}

	// Apply per-request JA3/HTTP2FP overrides only when the value has changed.
	if req.JA3 != "" && req.Browser == "" && req.JA3 != sw.LastJA3 {
		sw.Session.ApplyJa3(req.JA3, sw.Browser)
		sw.LastJA3 = req.JA3
	}
	if req.HTTP2FP != "" && req.Browser == "" && req.HTTP2FP != sw.LastHTTP2FP {
		sw.Session.ApplyHTTP2(req.HTTP2FP)
		sw.LastHTTP2FP = req.HTTP2FP
	}

	// Only call SetProxy when the proxy actually changes.
	if req.Proxy != "" && req.Proxy != sw.CurrentProxy {
		fmt.Fprintf(os.Stderr, "[sidecar] SetProxy: %q\n", req.Proxy)
		if err := sw.Session.SetProxy(req.Proxy); err != nil {
			fmt.Fprintf(os.Stderr, "[sidecar] SetProxy error for %q: %v\n", req.Proxy, err)
		} else {
			sw.CurrentProxy = req.Proxy
		}
	} else if req.Proxy == "" && sw.CurrentProxy != "" {
		sw.Session.ClearProxy()
		sw.CurrentProxy = ""
	}

	sw.mu.Unlock()

	// Build ordered headers
	var orderedHeaders azuretls.OrderedHeaders
	for _, h := range req.Headers {
		if len(h) == 2 {
			orderedHeaders = append(orderedHeaders, []string{h[0], h[1]})
		}
	}

	method := strings.ToUpper(req.Method)
	if method == "" {
		method = "GET"
	}

	httpReq := &azuretls.Request{
		Method:         method,
		Url:            req.URL,
		OrderedHeaders: orderedHeaders,
	}

	// Timeout is set per-request on the Request struct, not on the Session.
	// sw.Session.SetTimeout() mutates shared session state and races when two
	// goroutines use the same session concurrently — last write wins, causing
	// one request to run with the wrong timeout. azuretls.Request.TimeOut is
	// applied to that request only and is goroutine-safe.
	if req.Timeout > 0 {
		httpReq.TimeOut = time.Duration(req.Timeout) * time.Millisecond
	}

	if req.Body != "" && method != "GET" {
		httpReq.Body = req.Body
	}

	if req.FollowRedirects != nil && !*req.FollowRedirects {
		httpReq.DisableRedirects = true
	} else if req.MaxRedirects > 0 {
		httpReq.MaxRedirects = uint(req.MaxRedirects)
	}

	// Wrap the entire Do() — including proxy CONNECT tunnel — in a context timeout.
	// azuretls.Request.TimeOut applies to TLS handshake and response header phases
	// but NOT to the initial proxy TCP connection (net.Dialer inside proxyDialer has
	// no timeout). A context deadline bounds all phases end-to-end.
	timeout := time.Duration(req.Timeout) * time.Millisecond
	if timeout <= 0 {
		timeout = 30 * time.Second // fallback if block has no timeout set
	}
	reqCtx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()
	httpReq.SetContext(reqCtx)

	start := time.Now()
	resp, err := sw.Session.Do(httpReq)

	elapsed := time.Since(start).Milliseconds()

	if err != nil {
		// Enrich error message for common failure modes
		errMsg := err.Error()
		enriched := enrichError(errMsg)
		sendResponse(SidecarResponse{
			ID:       req.ID,
			Error:    enriched,
			TimingMs: elapsed,
		})
		return
	}

	// Extract response headers
	headers := make(map[string]string)
	for key, values := range resp.Header {
		headers[key] = strings.Join(values, "; ")
	}

	// Extract cookies from the cookie jar
	cookies := make(map[string]string)
	if sw.Session.CookieJar != nil {
		parsedURL, _ := url.Parse(req.URL)
		if parsedURL != nil {
			for _, cookie := range sw.Session.CookieJar.Cookies(parsedURL) {
				cookies[cookie.Name] = cookie.Value
			}
		}
	}

	finalURL := req.URL
	if resp.Url != "" {
		finalURL = resp.Url
	}

	// Capture the actual request headers sent by azuretls when requested.
	// azuretls.Request.HttpRequest is the underlying *http.Request which contains
	// all headers merged from the browser profile and any custom headers.
	var requestHeaders map[string]string
	if req.ReturnRequestHeaders && resp.Request != nil && resp.Request.HttpRequest != nil {
		requestHeaders = make(map[string]string, len(resp.Request.HttpRequest.Header))
		for key, values := range resp.Request.HttpRequest.Header {
			requestHeaders[key] = strings.Join(values, "; ")
		}
	}

	sendResponse(SidecarResponse{
		ID:             req.ID,
		Status:         resp.StatusCode,
		Headers:        headers,
		RequestHeaders: requestHeaders,
		Body:           string(resp.Body),
		Cookies:        cookies,
		FinalURL:       finalURL,
		TimingMs:       elapsed,
	})
}

// defaultJA3ForBrowser returns a standard JA3 string for the given browser profile.
// These are real-world fingerprints used to build a base for cipher suite overrides.
func defaultJA3ForBrowser(browser string) string {
	switch strings.ToLower(browser) {
	case "firefox":
		return "771,4865-4867-4866-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21,29-23-24,0"
	case "safari", "ios":
		return "771,4865-4866-4867-49196-49195-52393-49200-49199-52392-49162-49161-49172-49171-157-156-53-47-49160-49170-10,0-23-65281-10-11-16-5-13-18-51-45-43-27,29-23-24-25,0"
	case "edge":
		return "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21,29-23-24,0"
	default: // chrome
		return "771,4865-4866-4867-49195-49199-49196-49200-52393-52392-49171-49172-156-157-47-53,0-23-65281-10-11-35-16-5-13-18-51-45-43-27-21,29-23-24,0"
	}
}

// applyCustomCiphers overrides the cipher suites in a session by injecting a custom
// cipher list into the browser's default JA3 and re-applying the fingerprint.
// ciphers should be dash-separated IANA decimal IDs, e.g. "4865-4866-4867-49195-49199"
func applyCustomCiphers(session *azuretls.Session, browser, ciphers string) error {
	baseJA3 := defaultJA3ForBrowser(browser)
	parts := strings.SplitN(baseJA3, ",", 5)
	if len(parts) != 5 {
		return fmt.Errorf("invalid base JA3")
	}
	// Replace the cipher field (index 1) with the custom cipher list
	parts[1] = strings.TrimSpace(ciphers)
	customJA3 := strings.Join(parts, ",")
	return session.ApplyJa3(customJA3, browser)
}

// enrichError adds context to common error messages to aid debugging.
// Order matters: more specific patterns must come before general ones.
func enrichError(msg string) string {
	switch {
	// ── Proxy-layer errors (proxyconnect prefix = Go's HTTP proxy tunnel) ────
	case strings.Contains(msg, "proxyconnect") && strings.Contains(msg, "connection refused"):
		return msg + " [Proxy refused connection — the proxy host:port is wrong, dead, or not accepting connections. Check proxy settings]"
	case strings.Contains(msg, "proxyconnect") && strings.Contains(msg, "i/o timeout"):
		return msg + " [Proxy connect timed out — proxy is slow or unreachable. Check proxy host:port and consider increasing timeout]"
	case strings.Contains(msg, "proxyconnect") && strings.Contains(msg, "no such host"):
		return msg + " [Proxy hostname not found — DNS cannot resolve the proxy host. Check proxy format: scheme://host:port]"
	case strings.Contains(msg, "proxyconnect"):
		return msg + " [Proxy tunnel error — could not establish CONNECT tunnel through proxy. Check proxy credentials and format]"

	// ── Context / timeout errors ─────────────────────────────────────────────
	case strings.Contains(msg, "context deadline exceeded"):
		return msg + " [Request timed out — consider increasing the HTTP block timeout. If using a proxy, the proxy connect may be slow]"
	case strings.Contains(msg, "context canceled"):
		return msg + " [Request was cancelled — job stopped or timeout reached]"

	// ── TLS errors ───────────────────────────────────────────────────────────
	case strings.Contains(msg, "SEC_E_ILLEGAL_MESSAGE"):
		return msg + " [TLS: Server rejected handshake — try a different JA3 fingerprint or set ssl_verify=false for debugging]"
	case strings.Contains(msg, "tls: handshake failure") || strings.Contains(msg, "handshake failure"):
		return msg + " [TLS handshake failed — server may require specific cipher suites or SNI. Try a Chrome JA3 fingerprint]"
	case strings.Contains(msg, "certificate") && strings.Contains(msg, "expired"):
		return msg + " [TLS: Server certificate expired — set ssl_verify=false to bypass (testing only)]"
	case strings.Contains(msg, "certificate signed by unknown authority"):
		return msg + " [TLS: Self-signed/unknown CA — set ssl_verify=false to bypass (testing only)]"

	// ── Windows socket exhaustion ────────────────────────────────────────────
	case strings.Contains(msg, "Only one usage of each socket address") ||
		strings.Contains(msg, "WSAEADDRINUSE") ||
		strings.Contains(msg, "connectex") && strings.Contains(msg, "normally permitted"):
		return msg + " [Windows port exhaustion — ephemeral ports (49152–65535) are full. Reduce thread count, or the fix has been applied (SO_LINGER=0). Restart the job]"

	// ── Connection errors (target server, not proxy) ─────────────────────────
	case strings.Contains(msg, "EOF") || strings.Contains(msg, "empty response"):
		return msg + " [Empty response — server closed connection. Common causes: TLS fingerprint rejected, IP banned, proxy issue]"
	case strings.Contains(msg, "connection refused"):
		return msg + " [Target server refused connection — port may be closed or server is down. Verify the URL is correct]"
	case strings.Contains(msg, "no route to host") || strings.Contains(msg, "host unreachable") || strings.Contains(msg, "network is unreachable"):
		return msg + " [Network unreachable — check proxy settings or network connectivity]"
	case strings.Contains(msg, "no such host"):
		return msg + " [DNS error — hostname not found. Check the URL for typos]"
	case strings.Contains(msg, "i/o timeout"):
		return msg + " [I/O timeout — server is not responding. Consider increasing the HTTP block timeout]"
	default:
		return msg
	}
}


