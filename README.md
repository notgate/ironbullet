# ironbullet

A pipeline-based HTTP automation toolkit with a visual block editor. Build, test, and run complex request sequences through an intuitive drag-and-drop interface — no code required.

ironbullet is a ground-up rewrite designed for performance, stability, and transparency. If you've used tools like SilverBullet, you already know the frustration: unstable releases, premium pricing for what amounts to modified open-source code built on top of OpenBullet, and a general lack of care for the end user. We built ironbullet because the community deserves better than paying for resold open-source work wrapped in a subscription model.

That said, we want to give credit where it's due. The OpenBullet2 project and its community have been an invaluable resource. Much of the config format compatibility and block design philosophy in ironbullet draws from the patterns OB2 established. This project wouldn't exist without that foundation, and we appreciate every contributor who made it possible.

## Features

- **Visual Block Editor** — Drag-and-drop pipeline builder with 50+ block types across requests, parsing, control flow, browser automation, and more
- **Config Compatibility** — Import existing `.opk`, `.svb`, `.loli`, and OpenBullet JSON configs directly
- **Code Generation** — Export pipelines to standalone Rust programs using `wreq` for native performance
- **Multi-Tab Workspace** — Work on multiple configs simultaneously with full undo/redo support
- **Built-in Runner** — Multi-threaded execution engine with proxy rotation, retry logic, and real-time statistics
- **Job Manager** — Queue and manage multiple runner jobs with independent data sources and output targets
- **Browser Automation** — Chromium-based browser blocks for sites that require JavaScript rendering
- **Plugin System** — Extend functionality with native Rust plugins compiled at runtime
- **Protocol Support** — HTTP, TCP, UDP, FTP, SSH, IMAP, SMTP, and POP3 blocks
- **Bypass Blocks** — Built-in support for CAPTCHA solvers, Cloudflare bypass, DataDome, and CSRF extraction
- **Security Scanner** — Automatic detection of suspicious patterns when importing third-party configs
- **Auto-Updater** — Checks for new releases and updates in-place

## Installation

Download the latest release from the [Releases](https://github.com/ZeraTS/ironbullet/releases) page.

### Building from Source

**Prerequisites:**
- Rust 1.75+ with `cargo`
- Node.js 20+ with `npm`

```bash
# Build the frontend
cd gui
npm install
npm run build

# Build the application
cd ..
cargo build --release
```

The compiled binary will be at `target/release/ironbullet.exe`.

## CLI Usage

ironbullet can also run configs from the command line:

```bash
ironbullet --config path/to/config.opk --wordlist data.txt --threads 100
```

Run `ironbullet --help` for all available options.

## Architecture

- **Backend** — Rust with Tokio async runtime, wry/tao for the native window
- **Frontend** — SvelteKit with Tailwind CSS, served via custom protocol
- **Execution Engine** — Async block pipeline with per-request context isolation
- **Plugin ABI** — C-compatible FFI for loading `.dll`/`.so` plugins at runtime

## Config Format

ironbullet uses a JSON-based pipeline format. Configs from other tools can be imported through the GUI or CLI:

| Format | Extension | Support |
|--------|-----------|---------|
| OpenBullet Pack | `.opk` | Full |
| SilverBullet | `.svb` | Full |
| LoliCode | `.loli` | Full |
| OpenBullet JSON | `.json` | Full |

## License

This project is licensed under the [MIT License](LICENSE).

## Contributing

Contributions are welcome. Please open an issue first to discuss what you'd like to change.
