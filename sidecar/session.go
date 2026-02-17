package main

import (
	"fmt"
	"net/http"
	"net/url"
	"strings"
	"time"

	"github.com/Noooste/azuretls-client"
)

type SessionWrapper struct {
	Session *azuretls.Session
	Browser string
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
			ID:      req.ID + "_init",
			Action:  "new_session",
			Session: req.Session,
			Browser: req.Browser,
			Proxy:   req.Proxy,
		})
		sessionsMu.RLock()
		sw, ok = sessions[req.Session]
		sessionsMu.RUnlock()
		if !ok {
			sendError(req.ID, "Failed to auto-create session")
			return
		}
	}

	// Set per-request proxy if different
	if req.Proxy != "" {
		sw.Session.SetProxy(req.Proxy)
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

	resp, err := sw.Session.Do(httpReq)

	elapsed := time.Since(start).Milliseconds()

	if err != nil {
		sendResponse(SidecarResponse{
			ID:       req.ID,
			Error:    err.Error(),
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
