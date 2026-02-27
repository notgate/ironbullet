package main

import (
	"fmt"
	"net/http"
	"net/url"
	"os"
	"strings"
	"time"

	"github.com/Noooste/azuretls-client"
)

type SessionWrapper struct {
	Session      *azuretls.Session
	Browser      string
	// CurrentProxy tracks the proxy currently configured on this session.
	// SetProxy is expensive (rebuilds transport state in azuretls), so we
	// skip the call when the proxy hasn't changed since the last request.
	CurrentProxy string
}

func handleNewSession(req SidecarRequest) {
	session := azuretls.NewSession()

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

	sessionsMu.Lock()
	sessions[req.Session] = &SessionWrapper{
		Session: session,
		Browser: browser,
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
		// Auto-create session if it doesn't exist
		handleNewSession(SidecarRequest{
			ID:        req.ID + "_init",
			Action:    "new_session",
			Session:   req.Session,
			Browser:   req.Browser,
			Proxy:     req.Proxy,
			SslVerify: req.SslVerify,
		})
		sessionsMu.RLock()
		sw, ok = sessions[req.Session]
		sessionsMu.RUnlock()
		if !ok {
			sendError(req.ID, "Failed to auto-create session")
			return
		}
	}

	// Only call SetProxy when the proxy actually changes.
	// azuretls rebuilds internal transport state on SetProxy, which tears down
	// any keep-alive connections — calling it redundantly on every request
	// within the same pipeline execution (same proxy, multiple HTTP blocks)
	// forces a fresh TCP+TLS handshake each time and kills throughput.
	if req.Proxy != "" && req.Proxy != sw.CurrentProxy {
		if err := sw.Session.SetProxy(req.Proxy); err != nil {
			// Non-fatal: log and continue with previous proxy config
			fmt.Fprintf(os.Stderr, "[sidecar] SetProxy error for %s: %v\n", req.Proxy, err)
		} else {
			sw.CurrentProxy = req.Proxy
		}
	} else if req.Proxy == "" && sw.CurrentProxy != "" {
		// Proxy explicitly cleared — reset to direct connection
		sw.Session.SetProxy("")
		sw.CurrentProxy = ""
	}

	// Build ordered headers
	var orderedHeaders azuretls.OrderedHeaders
	for _, h := range req.Headers {
		if len(h) == 2 {
			orderedHeaders = append(orderedHeaders, []string{h[0], h[1]})
		}
	}

	// Set timeout
	if req.Timeout > 0 {
		sw.Session.SetTimeout(time.Duration(req.Timeout) * time.Millisecond)
	}

	start := time.Now()

	method := strings.ToUpper(req.Method)
	if method == "" {
		method = "GET"
	}

	httpReq := &azuretls.Request{
		Method:         method,
		Url:            req.URL,
		OrderedHeaders: orderedHeaders,
	}

	if req.Body != "" && method != "GET" {
		httpReq.Body = req.Body
	}

	// Bug-fix: follow_redirects and max_redirects were previously ignored.
	// Use per-request fields so session-level defaults aren't clobbered.
	if req.FollowRedirects != nil && !*req.FollowRedirects {
		httpReq.DisableRedirects = true // returns 3xx directly without following
	} else if req.MaxRedirects > 0 {
		httpReq.MaxRedirects = uint(req.MaxRedirects)
	}

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

	sendResponse(SidecarResponse{
		ID:       req.ID,
		Status:   resp.StatusCode,
		Headers:  headers,
		Body:     string(resp.Body),
		Cookies:  cookies,
		FinalURL: finalURL,
		TimingMs: elapsed,
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

// enrichError adds context to common error messages to aid debugging
func enrichError(msg string) string {
	switch {
	case strings.Contains(msg, "SEC_E_ILLEGAL_MESSAGE"):
		return msg + " [TLS: Server rejected handshake — try enabling a different JA3 fingerprint or set ssl_verify=false for debugging]"
	case strings.Contains(msg, "tls: handshake failure") || strings.Contains(msg, "handshake failure"):
		return msg + " [TLS handshake failed — server may require specific cipher suites or SNI. Try a Chrome JA3 fingerprint]"
	case strings.Contains(msg, "certificate") && strings.Contains(msg, "expired"):
		return msg + " [TLS: Server certificate expired — set ssl_verify=false to bypass (testing only)]"
	case strings.Contains(msg, "certificate signed by unknown authority"):
		return msg + " [TLS: Self-signed/unknown CA — set ssl_verify=false to bypass (testing only)]"
	case strings.Contains(msg, "EOF") || strings.Contains(msg, "empty response"):
		return msg + " [Empty response — server closed connection. Common causes: TLS fingerprint rejected, IP banned, proxy issue]"
	case strings.Contains(msg, "connection refused"):
		return msg + " [Connection refused — server is down or port is blocked]"
	case strings.Contains(msg, "no such host") || strings.Contains(msg, "i/o timeout"):
		return msg + " [DNS/network error — check URL and proxy settings]"
	default:
		return msg
	}
}

func getCookies(header http.Header) map[string]string {
	cookies := make(map[string]string)
	for _, c := range header["Set-Cookie"] {
		parts := strings.SplitN(c, "=", 2)
		if len(parts) == 2 {
			name := strings.TrimSpace(parts[0])
			value := strings.SplitN(parts[1], ";", 2)[0]
			cookies[name] = strings.TrimSpace(value)
		}
	}
	return cookies
}
