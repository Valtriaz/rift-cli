# RIFT-CLI

A fast, terminal-native web browser built in Rust with Ratatui and Crossterm.

## Features

- ⚡ **Lightweight & Fast:** Instant startup and ultra-low memory footprint using native Rust HTTP fetching (`reqwest`).
- 🖥️ **TUI Layout & Box Model:** Renders headers, footers, cards, sidebars, buttons, blockquotes, and code blocks using clean Unicode borders.
- 🔗 **Smart Navigation:** Tab/Shift+Tab keyboard link selection with relative URL resolution.
- 📜 **Full noscript & SPA Support:** Automatically parses `<noscript>` fallback fragments and extracts OpenGraph / meta descriptions for client-side web apps.
- 🌐 **Headless Chromium Mode (Optional):** Render complex JavaScript single-page apps (SPAs) and dynamic Web apps using headless Chromium via `--js` or dynamically toggled with `Ctrl+J`.

## Usage

```bash
# Launch RIFT-CLI
rift-cli

# Open a website directly
rift-cli https://example.com

# Open with JavaScript execution enabled (via headless Chromium)
rift-cli --js https://smitronix.dev
```

### Options

| Flag | Description |
| :--- | :--- |
| `[URL]` | Initial URL to navigate to upon startup |
| `--js`, `--chromium` | Enable headless Chromium JavaScript rendering |
| `-h`, `--help` | Show command-line help and keybindings |

### Keybindings

| Key | Action |
| :--- | :--- |
| `Enter` | Navigate to address in URL bar, or follow selected link |
| `Tab` | Select next link on page |
| `Shift + Tab` | Select previous link on page |
| `Esc` | Clear selected link |
| `↑` / `↓` | Scroll up / down |
| `PgUp` / `PgDn` | Scroll page up / down |
| `Home` / `End` | Jump to top / bottom of page |
| `Ctrl + J` | Toggle JavaScript (Chromium) mode on / off |
| `Ctrl + Q` | Quit RIFT-CLI |

## Installation & Build

```bash
cargo build --release
./target/release/rift-cli
```

## License

MIT