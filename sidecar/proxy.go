package main

// Proxy configuration is handled directly via session.SetProxy()
// in session.go. This file provides utility functions for proxy
// format validation and conversion.

import (
	"fmt"
	"strings"
)

// ProxyInfo holds parsed proxy details
type ProxyInfo struct {
	Type     string // http, https, socks4, socks5
	Host     string
	Port     string
	Username string
	Password string
}

// ParseProxyString parses various proxy formats:
// - http://host:port
// - socks5://user:pass@host:port
// - TYPE:HOST:PORT:USER:PASS
// - HOST:PORT
func ParseProxyString(proxy string) (*ProxyInfo, error) {
	// URL format
	for _, prefix := range []string{"http://", "https://", "socks4://", "socks5://"} {
		if strings.HasPrefix(proxy, prefix) {
			proxyType := strings.TrimSuffix(prefix, "://")
			rest := strings.TrimPrefix(proxy, prefix)

			info := &ProxyInfo{Type: proxyType}

			// Check for user:pass@host:port
			if atIdx := strings.LastIndex(rest, "@"); atIdx >= 0 {
				userPass := rest[:atIdx]
				hostPort := rest[atIdx+1:]

				parts := strings.SplitN(userPass, ":", 2)
				info.Username = parts[0]
				if len(parts) > 1 {
					info.Password = parts[1]
				}

				hpParts := strings.SplitN(hostPort, ":", 2)
				info.Host = hpParts[0]
				if len(hpParts) > 1 {
					info.Port = hpParts[1]
				}
			} else {
				hpParts := strings.SplitN(rest, ":", 2)
				info.Host = hpParts[0]
				if len(hpParts) > 1 {
					info.Port = hpParts[1]
				}
			}

			return info, nil
		}
	}

	// Colon-separated format
	parts := strings.Split(proxy, ":")
	switch len(parts) {
	case 2: // HOST:PORT
		return &ProxyInfo{
			Type: "http",
			Host: parts[0],
			Port: parts[1],
		}, nil
	case 4: // HOST:PORT:USER:PASS
		return &ProxyInfo{
			Type:     "http",
			Host:     parts[0],
			Port:     parts[1],
			Username: parts[2],
			Password: parts[3],
		}, nil
	case 5: // TYPE:HOST:PORT:USER:PASS
		return &ProxyInfo{
			Type:     strings.ToLower(parts[0]),
			Host:     parts[1],
			Port:     parts[2],
			Username: parts[3],
			Password: parts[4],
		}, nil
	}

	return nil, fmt.Errorf("unrecognized proxy format: %s", proxy)
}

// ToURL converts a ProxyInfo to a URL string for azuretls
func (p *ProxyInfo) ToURL() string {
	auth := ""
	if p.Username != "" {
		auth = p.Username
		if p.Password != "" {
			auth += ":" + p.Password
		}
		auth += "@"
	}
	return fmt.Sprintf("%s://%s%s:%s", p.Type, auth, p.Host, p.Port)
}
