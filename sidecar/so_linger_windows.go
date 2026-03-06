//go:build windows

package main

import (
	"syscall"
	"unsafe"
)

// soLingerZero sets SO_LINGER with l_onoff=1, l_linger=0 on the socket.
// On Windows, this causes the OS to send TCP RST on close instead of going
// through the TIME_WAIT state. TIME_WAIT holds the local ephemeral port for
// ~4 minutes (2×MSL = 2×120s = 240s by default on Windows), and with 100+
// threads each making rapid connections, the ~16k ephemeral port range
// (49152–65535) fills up and new connections fail with:
//   "connectex: Only one usage of each socket address (protocol/network
//    address/port) is normally permitted."
//
// Setting SO_LINGER=0 releases ports immediately and eliminates this error.
// The RST-on-close is acceptable for outbound HTTP connections where we don't
// need a graceful 4-way FIN handshake.
func soLingerZero(network, address string, c syscall.RawConn) error {
	// Windows linger struct: {l_onoff uint16, l_linger uint16}
	type lingerStruct struct {
		Onoff  uint16
		Linger uint16
	}
	linger := lingerStruct{Onoff: 1, Linger: 0}

	var setSockOptErr error
	err := c.Control(func(fd uintptr) {
		setSockOptErr = syscall.Setsockopt(
			syscall.Handle(fd),
			syscall.SOL_SOCKET,
			syscall.SO_LINGER,
			(*byte)(unsafe.Pointer(&linger)),
			int32(unsafe.Sizeof(linger)),
		)
	})
	if err != nil {
		return err
	}
	return setSockOptErr
}
