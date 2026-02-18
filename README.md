# Ironbullet

<div align="center">

![Ironbullet Logo](data/IMGS/notextlogo.png)

**Visual pipeline builder for HTTP automation and credential checking**

[![GitHub release](https://img.shields.io/github/v/release/ZeraTS/ironbullet)](https://github.com/ZeraTS/ironbullet/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

[Features](#features) • [Installation](#installation) • [Quick Start](#quick-start) • [Documentation](#documentation) • [Contributing](#contributing)

</div>

---

## Overview

Ironbullet is a modern, visual pipeline builder designed for HTTP automation, credential checking, and web scraping workflows. Build complex automation pipelines using a drag-and-drop block-based interface with 50+ specialized blocks for HTTP requests, data parsing, cryptography, browser automation, and more.

### Why Ironbullet?

- **Visual Pipeline Editor** - Build workflows visually with drag-and-drop blocks
- **50+ Block Types** - HTTP, parsing, crypto, browser, protocols, bypass techniques
- **Multi-threaded Execution** - Run pipelines at scale with configurable thread pools
- **Debug Mode** - Step through execution with request/response inspection
- **Code Generation** - Export pipelines as standalone Rust code
- **Plugin System** - Extend functionality with custom .dll plugins
- **OpenBullet Compatible** - Import .svb, .opk, and .loliScript configs

## Features

### 🎨 Visual Pipeline Editor
- Drag-and-drop block-based workflow builder
- 50+ block types: HTTP, parsing, crypto, browser, protocols, bypass
- Block search/filter with Ctrl+F and match highlighting
- Collapse/expand container blocks (IfElse, Loop, Group)
- Pipeline minimap with viewport tracking
- Multi-select with rubber band selection
- Block templates for reusable patterns
- Undo/redo with full history
- Variable preview mode with inline resolution

### 🚀 Enhanced Variable System (v0.2.0)
- **Three input modes** on all text fields:
  - **RAW** - Literal text (no interpolation)
  - **EMBED** - Variable interpolation with `<variable>` or `{{variable}}` syntax
  - **VAR** - Dropdown selector for pipeline variables
- Color-coded mode badges
- Auto-populated variable list
- Available across all block types

### 🔧 New Function Blocks (v0.2.0)
- **ByteArray** - Binary data manipulation (hex, base64, UTF-8)
- **Constants** - Define reusable constants
- **Dictionary** - Key-value data structures
- **FloatFunction** - Floating-point math
- **IntegerFunction** - Integer arithmetic
- **TimeFunction** - Timezone conversion and duration calculations
- **GenerateGUID** - UUID generation (v1, v4, v5)
- **PhoneCountry** - Extract country codes from phone numbers
- **LambdaParser** - Parse text with lambda expressions

### 🏃 Runner & Debugging
- Multi-threaded runner with configurable thread count
- Debug mode with step-through execution
- Network log with request/response inspection
- Proxy support with ban detection and rotation
- Proxy health checking
- Job manager for queuing multiple runs
- Hits Database panel with filtering and management

### 📦 Config Management
- Multi-tab config editing with unsaved change tracking
- Save/load .json pipeline configs
- Import OpenBullet .svb, .opk, and .loliScript formats
- Collections folder for quick access
- Recent configs list on startup
- Security scanner for imported configs

### 🔌 Code Generation & Plugins
- Export pipeline as standalone Rust code
- Plugin system with .dll hot-loading
- Built-in Plugin Builder with code generation
- Akamai v3 sensor generation block

### 🎨 UI & Settings
- Dark theme with skeuomorphic controls
- Scalable font size affecting all UI
- Configurable zoom level
- Adjustable panel widths and heights
- Custom window chrome
- Auto-updater with in-app download
- Block documentation panel (F1)
- Comprehensive keyboard shortcuts

## Installation

### Download Release

1. Download the latest release from [Releases](https://github.com/ZeraTS/ironbullet/releases)
2. Extract the archive to a folder
3. Run `ironbullet.exe`

**Note:** The sidecar binary (`reqflow-sidecar.exe`) must be in the same directory as the main executable.

### Build from Source

**Prerequisites:**
- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Node.js 20+ ([Install Node](https://nodejs.org/))

```bash
# Clone repository
git clone https://github.com/ZeraTS/ironbullet.git
cd ironbullet

# Build backend
cargo build --release

# Build frontend
cd gui
npm install
npm run build

# Build sidecar
cd ../sidecar
go build -o reqflow-sidecar.exe

# Run
cd ..
./target/release/ironbullet.exe
```

## Quick Start

### Creating Your First Pipeline

1. **Launch Ironbullet**
2. **Add an HTTP Request block** from the palette (left panel)
3. **Configure the request:**
   - Set URL (e.g., `https://example.com/login`)
   - Add headers, body, cookies as needed
4. **Add a Parser block** to extract data:
   - JSON Parser for JSON responses
   - Regex Parser for pattern matching
   - CSS Selector for HTML parsing
5. **Add a KeyCheck block** to validate results
6. **Test with Debug Mode** (F5):
   - Enter test credentials
   - Click "Debug Run"
   - Inspect request/response in panels

### Example: Login Checker

```
1. HTTP Request
   URL: https://example.com/api/login
   Method: POST
   Body: {"username":"<USER>","password":"<PASS>"}

2. JSON Parser
   Path: $.token
   Output: TOKEN

3. KeyCheck
   Condition: TOKEN exists
   Success: Valid credentials
   Fail: Invalid credentials
```

## Documentation

### Block Categories

- **HTTP Requests** - GET, POST, custom headers, cookies, authentication
- **Parsing** - JSON, Regex, CSS, XPath, Cookie extraction
- **Functions** - String, List, Crypto, Date, Math operations
- **Control Flow** - IfElse, Loop, Delay, Set Variable
- **Browser** - Selenium automation, screenshots, JavaScript execution
- **Protocols** - TCP, UDP, FTP, SSH, IMAP, SMTP, POP
- **Bypass** - Captcha solving, Cloudflare bypass, CSRF token extraction
- **Utilities** - Logging, scripting, random data generation

### Keyboard Shortcuts

- `Ctrl+F` - Search blocks
- `Ctrl+C/V` - Copy/paste blocks
- `Ctrl+Z/Y` - Undo/redo
- `F1` - Block documentation
- `F5` - Debug run
- `Ctrl+S` - Save config
- `Delete` - Remove selected blocks

### Variable System

Variables can be referenced using two syntaxes:
- `<VARIABLE_NAME>` - Classic syntax
- `{{VARIABLE_NAME}}` - Modern syntax

Built-in variables:
- `input.DATA` - Current data line
- `input.USER` - Username (if using combo format)
- `input.PASS` - Password (if using combo format)
- `data.SOURCE` - HTTP response body
- `data.HEADERS` - HTTP response headers
- `data.COOKIES` - HTTP response cookies

## Contributing

Contributions are welcome! Please feel free to submit pull requests or open issues for bugs and feature requests.

### Development Setup

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Test thoroughly
5. Commit with descriptive messages
6. Push to your fork
7. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by OpenBullet and SilverBullet
- Built with Rust, SvelteKit, and Tauri
- Uses Monaco Editor for code editing

## Support

- **Issues:** [GitHub Issues](https://github.com/ZeraTS/ironbullet/issues)
- **Discussions:** [GitHub Discussions](https://github.com/ZeraTS/ironbullet/discussions)

---

<div align="center">
Made with ❤️ by the Ironbullet team
</div>
