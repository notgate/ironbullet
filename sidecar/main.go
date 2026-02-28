package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"sync"
)

// SidecarRequest from Rust
type SidecarRequest struct {
	ID              string     `json:"id"`
	Action          string     `json:"action"`
	Session         string     `json:"session"`
	Method          string     `json:"method,omitempty"`
	URL             string     `json:"url,omitempty"`
	Headers         [][]string `json:"headers,omitempty"`
	Body            string     `json:"body,omitempty"`
	Timeout         int64      `json:"timeout,omitempty"`
	Proxy           string     `json:"proxy,omitempty"`
	Browser         string     `json:"browser,omitempty"`
	JA3             string     `json:"ja3,omitempty"`
	HTTP2FP         string     `json:"http2fp,omitempty"`
	FollowRedirects *bool      `json:"follow_redirects,omitempty"`
	MaxRedirects    int64      `json:"max_redirects,omitempty"`
	// SslVerify: nil = default (verify), false = skip TLS verification
	SslVerify       *bool      `json:"ssl_verify,omitempty"`
	// CustomCiphers: optional dash-separated cipher suite IDs to override the browser profile's default.
	// e.g. "4865-4866-4867-49195-49199-49196-49200-52393-52392"
	// Cipher IDs are IANA decimal values. Applied via azuretls JA3 override.
	CustomCiphers          string     `json:"custom_ciphers,omitempty"`
	// ReturnRequestHeaders: when true, the response will include the actual request headers
	// sent by azuretls. Used by the Site Inspector to display the full request.
	ReturnRequestHeaders   bool       `json:"return_request_headers,omitempty"`
}

// SidecarResponse to Rust
type SidecarResponse struct {
	ID             string            `json:"id"`
	Status         int               `json:"status"`
	Headers        map[string]string `json:"headers,omitempty"`
	// RequestHeaders captures the actual HTTP headers sent by azuretls on the wire.
	// Only populated when ReturnRequestHeaders is true in the request.
	RequestHeaders map[string]string `json:"request_headers,omitempty"`
	Body           string            `json:"body"`
	Cookies        map[string]string `json:"cookies,omitempty"`
	FinalURL       string            `json:"final_url"`
	Error          string            `json:"error,omitempty"`
	TimingMs       int64             `json:"timing_ms"`
}

var (
	sessions   = make(map[string]*SessionWrapper)
	sessionsMu sync.RWMutex
	outputMu   sync.Mutex
)

func main() {
	scanner := bufio.NewScanner(os.Stdin)
	scanner.Buffer(make([]byte, 0, 10*1024*1024), 10*1024*1024) // 10MB buffer

	for scanner.Scan() {
		line := scanner.Text()
		if line == "" {
			continue
		}

		var req SidecarRequest
		if err := json.Unmarshal([]byte(line), &req); err != nil {
			sendError("", fmt.Sprintf("Invalid JSON: %v", err))
			continue
		}

		go handleRequest(req)
	}
}

func handleRequest(req SidecarRequest) {
	switch req.Action {
	case "new_session":
		handleNewSession(req)
	case "close_session":
		handleCloseSession(req)
	case "request":
		handleHTTPRequest(req)
	case "set_proxy":
		handleSetProxy(req)
	case "set_browser":
		handleSetBrowser(req)
	case "ping":
		sendResponse(SidecarResponse{
			ID:     req.ID,
			Status: 200,
			Body:   "pong",
		})
	default:
		sendError(req.ID, fmt.Sprintf("Unknown action: %s", req.Action))
	}
}

func sendResponse(resp SidecarResponse) {
	data, err := json.Marshal(resp)
	if err != nil {
		return
	}
	outputMu.Lock()
	fmt.Fprintln(os.Stdout, string(data))
	os.Stdout.Sync()
	outputMu.Unlock()
}

func sendError(id string, errMsg string) {
	sendResponse(SidecarResponse{
		ID:    id,
		Error: errMsg,
	})
}
