# IronBullet v0.5.3

**Release Date:** 2026-04-05
**Type:** Stable Release

---

## Key Fixes

### SOCKS5 Proxies in Jobs Mode (Issue #56)

SOCKS5 proxies previously worked in Debugger mode but failed in Jobs mode unless manually configured.

**What changed:**
- `host:port` entries are now auto-detected
- Common SOCKS5 and Shadowsocks ports are recognized automatically
- Explicit proxy schemes still take priority

**Result:** SOCKS5 proxies now work in Jobs mode without extra configuration.

---

### WebView2 Memory Leak with Shadowsocks

Long-running jobs with Shadowsocks caused CPU and memory usage to spike due to untracked background tasks.

**What changed:**
- All spawned tasks are now tracked and properly cleaned up
- Old or dead tasks are aborted before new ones start
- Prevents duplicate task buildup

**Result:** Stable performance during extended runs.

---

## Improvements

- Added a proxy parsing test suite covering multiple formats and edge cases
- Improved internal documentation and code comments

---

**Reminder:** This is a release candidate. Test before using in production.
