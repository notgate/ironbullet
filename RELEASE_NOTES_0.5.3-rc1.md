# IronBullet v0.5.3-rc1 Release Notes

**Release Date:** 2026-04-05
**Type:** Release Candidate 1
**Stability:** Pre-release for testing

---

## Critical Bug Fixes

### Issue #56: SOCKS5 Proxies Ignored in Jobs Mode

**Status:** FIXED

**Problem:**
- SOCKS5 proxies worked in Debugger mode but were ignored in Jobs mode
- When proxy files contained plain `host:port` entries (e.g., `127.0.0.1:1080`), they were incorrectly parsed as HTTP proxies
- Users had to manually specify `default_proxy_type: "socks5"` on every proxy source

**Root Cause:**
- Jobs mode parsed proxy files without scheme detection
- Default fallback was always HTTP for untyped entries
- Debugger mode accepted full proxy URLs directly from frontend

**Solution:**
- Added intelligent proxy type detection based on port numbers
- Common SOCKS5 ports (1080, 1081, 9050, 9150) now auto-detect as SOCKS5
- Shadowsocks local ports (1086, 1087, 1088) also auto-detect
- Explicit scheme prefixes still take precedence
- Maintains backward compatibility with existing proxy files

**Testing:**
```bash
# Before fix: treated as HTTP, connection fails
127.0.0.1:1080

# After fix: auto-detects as SOCKS5, works correctly
127.0.0.1:1080  -> socks5://127.0.0.1:1080
```

**Impact:** HIGH - Users with SOCKS5 proxies can now use Jobs mode without manual configuration

---

### WebView2 Memory Leak with Shadowsocks Proxy

**Status:** FIXED

**Problem:**
- CPU and memory usage increased to 100% during extended Jobs runs with Shadowsocks proxies
- WebView2 manager became unresponsive
- Each job start/stop cycle spawned new Shadowsocks server tasks that were never cleaned up
- Liveness probes re-spawned servers repeatedly when old ones died

**Root Cause:**
- Shadowsocks pool spawned `tokio::task` for each unique SS server
- Tasks ran indefinitely with no lifecycle management
- No cleanup mechanism when servers died or were no longer needed
- Task accumulation over multiple job runs

**Solution:**
- Track all spawned tasks with `JoinHandle` in the pool map
- Abort old tasks before spawning new ones for the same SS URL
- Added `shutdown_all()` function for application cleanup
- Dead servers now properly abort their tasks before re-spawn
- Prevent duplicate task spawning on race conditions

**Technical Details:**
```rust
// Before: fire-and-forget spawn
tokio::spawn(async move { server.run().await });

// After: tracked handle with abort capability
let task_handle = tokio::spawn(async move { server.run().await });
map.insert(ss_url, SsEntry { local_url, task_handle });

// Cleanup: abort task when no longer needed
entry.task_handle.abort();
```

**Impact:** CRITICAL - Prevents unbounded memory growth and ensures stable long-running jobs

---

## Code Quality Improvements

### Test Suite

Added comprehensive test suite for proxy parsing:
- `tests/proxy_parsing_test.rs` - 11 test cases covering all proxy formats
- Test fixtures for SOCKS5 and HTTP proxy files
- Validates auto-detection, explicit schemes, authentication, mixed types

**Run tests:**
```bash
cargo test proxy_parsing_test
cargo test --test proxy_parsing_test
```

### Documentation

- `BUGFIX_ANALYSIS.md` - Detailed technical analysis of both issues
- Inline code comments explaining port detection logic
- Release notes with testing examples

---

## Breaking Changes

**None.** This release is fully backward compatible.

---

## Upgrading

### From 0.5.2

No configuration changes required. Existing proxy files will work as before, with improved SOCKS5 detection as a bonus.

### Testing Your Setup

1. Verify SOCKS5 auto-detection:
   ```bash
   # Create test proxy file with plain host:port
   echo "127.0.0.1:1080" > test_socks5.txt

   # Use in Jobs mode - should now work without default_proxy_type
   ```

2. Monitor memory usage during extended runs:
   ```bash
   # Run a job with Shadowsocks for 10+ minutes
   # Memory should remain stable (no growth to 100%)
   ```

3. Check issue #56 test case:
   ```bash
   # Jobs mode with SOCKS5 proxy pointing to httpbin.org/ip
   # Should return proxy IP, not your real IP
   ```

---

## Known Issues

- Issue #51 (Drag-and-drop support) - Not implemented in this RC
- Frontend GUI requires rebuild: `cd gui && npm run build`

---

## Next Steps

**For Release Candidate Testing:**
1. Test with various SOCKS5 proxy providers
2. Run extended jobs (30+ minutes) with Shadowsocks to verify memory stability
3. Test mixed proxy files (HTTP + SOCKS5 + Shadowsocks)
4. Verify backward compatibility with existing configurations

**For Final 0.5.3 Release:**
- Address any RC1 bug reports
- Implement drag-and-drop support (Issue #51)
- Performance benchmarks
- Update documentation site

---

## Contributors

- Bug reports: ZeraTS/ironbullet GitHub issues
- Analysis and fixes: Claude Code (Anthropic)
- Testing: Community feedback on RC1

---

## Reporting Issues

Found a bug in RC1? Report it:
- GitHub: https://github.com/ZeraTS/ironbullet/issues
- Include: version (0.5.3-rc1), OS, proxy type, job configuration
- Attach: logs, screenshots if applicable

---

**Remember:** This is a Release Candidate. Test thoroughly before using in production.
