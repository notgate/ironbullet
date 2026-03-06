//go:build !windows

package main

import (
	"syscall"
)

// soLingerZero sets SO_LINGER with l_onoff=1, l_linger=0 on the socket.
// When the connection is closed, the OS sends RST immediately instead of
// going through the normal FIN/TIME_WAIT sequence. This releases the local
// ephemeral port immediately rather than holding it for ~4 minutes (2×MSL).
//
// On Linux/macOS this is desirable at high concurrency to avoid port exhaustion.
// The tradeoff: the remote end sees a connection reset rather than a clean close,
// which is fine for HTTP/1.1 (each request uses a new connection) and acceptable
// for our use case since we don't care about graceful teardown.
func soLingerZero(network, address string, c syscall.RawConn) error {
	var setSockOptErr error
	err := c.Control(func(fd uintptr) {
		setSockOptErr = syscall.SetsockoptLinger(int(fd), syscall.SOL_SOCKET, syscall.SO_LINGER,
			&syscall.Linger{Onoff: 1, Linger: 0})
	})
	if err != nil {
		return err
	}
	return setSockOptErr
}
