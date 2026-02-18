# Ironbullet

[![GitHub release](https://img.shields.io/github/v/release/ZeraTS/ironbullet)](https://github.com/ZeraTS/ironbullet/releases)
[![Downloads](https://img.shields.io/github/downloads/ZeraTS/ironbullet/total)](https://github.com/ZeraTS/ironbullet/releases)

Visual pipeline builder for HTTP automation and credential checking. Build complex automation workflows using a drag-and-drop block-based interface with 50+ specialized blocks.

## Features

- Visual block-based pipeline editor with drag-and-drop
- 50+ block types: HTTP, parsing, crypto, browser automation, protocols, bypass
- Multi-threaded execution with configurable thread pools
- Debug mode with request/response inspection
- Variable input system with RAW/EMBED/VAR modes supporting `<variable>` and `{{variable}}` syntax
- Proxy rotation with ban detection and health checking
- Export pipelines as standalone Rust code
- Plugin system with hot-loading support
- Import OpenBullet configs (.svb, .opk, .loliScript)

## Installation

Download the latest release from [Releases](https://github.com/ZeraTS/ironbullet/releases) and extract the archive. Run `ironbullet.exe` to start the application.

**Note:** The sidecar binary (`reqflow-sidecar.exe`) must be in the same directory.

## Quick Start

1. Launch Ironbullet
2. Add blocks from the palette (left panel)
3. Configure block settings (right panel)
4. Press F5 to debug with test data
5. Create a job to run against full datasets

## Block Categories

- **HTTP** - Requests with headers, cookies, authentication
- **Parsing** - JSON, Regex, CSS, XPath extraction
- **Functions** - String, List, Crypto, Math, Time operations
- **Control** - IfElse, Loop, variables, delays
- **Browser** - Selenium automation, screenshots, JavaScript
- **Protocols** - TCP, UDP, FTP, SSH, IMAP, SMTP
- **Bypass** - Captcha solving, Cloudflare, CSRF tokens
- **Utilities** - Logging, scripting, plugins

## Build from Source

Requirements: Rust 1.70+, Node.js 20+, Go 1.23+

```bash
git clone https://github.com/ZeraTS/ironbullet.git
cd ironbullet

# Build backend
cargo build --release

# Build frontend
cd gui && npm install && npm run build

# Build sidecar
cd ../sidecar && go build -o reqflow-sidecar.exe
```

## License

MIT License - see LICENSE file for details.
