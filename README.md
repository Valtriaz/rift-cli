# 🌐 RIFT-CLI

<p align="center">
  <strong>A fast, lightweight, keyboard-driven terminal web browser built in Rust.</strong>
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square" alt="License"></a>
  <a href="https://www.rust-lang.org/"><img src="https://img.shields.io/badge/Rust-2024%20Edition-orange.svg?style=flat-square" alt="Rust Edition"></a>
  <a href="https://github.com/Valtriaz/rift-cli/stargazers"><img src="https://img.shields.io/github/stars/Valtriaz/rift-cli?style=flat-square&color=yellow" alt="Stars"></a>
  <a href="https://github.com/Valtriaz/rift-cli/issues"><img src="https://img.shields.io/badge/PRs-Welcome-brightgreen.svg?style=flat-square" alt="PRs Welcome"></a>
  <a href="https://buymeacoffee.com/valtriaz"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?style=flat-square&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me A Coffee"></a>
</p>

---

## 🚀 Getting Started & How to Run

### 1. Prerequisites

Make sure you have **Rust & Cargo** installed (Rust 1.85+ / 2024 edition):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Clone & Run in One Step

Copy and paste this into your terminal to clone and start browsing immediately:

```bash
git clone https://github.com/Valtriaz/rift-cli.git && cd rift-cli && cargo run
```

### 3. Running with Different Options

```bash
# Launch empty browser
cargo run

# Open a website directly (defaults to https://)
cargo run

# View command-line options and shortcuts
cargo run -- --help
```

### 4. Install Globally as a System Command

To use `rift-cli` anywhere from any directory:

```bash
cargo install --path .
```

Once installed, simply run:

```bash
rift-cli
```

---

## ✨ Features

- ⚡ **Non-Blocking Background Fetching**
  - Network requests run asynchronously in a worker thread—the terminal **never freezes** during slow or unreachable connections.
  - An animated spinner `[⠋ Loading...]` shows progress while keeping the address bar fully responsive.
  - Press `Esc` at any moment to cancel loading, or type a new URL and press `Enter` to switch immediately.

- 🔗 **Numbered Links & Direct Jump Modal (`Ctrl+L` / `F2`)**
  - Every link on a webpage is indexed with a number (`[1]`, `[2]`, `[3]`).
  - Press **`Ctrl+L`** or **`F2`** to trigger a modal popup, type the link number, and jump directly to that page.
  - Press **`Tab`** or **`Shift+Tab`** to cycle links with **automatic viewport scrolling** so the selected link stays in view.

- ✏️ **Interactive Address Bar with Cursor Navigation**
  - Full back-and-forth cursor movement using `←` and `→` arrow keys.
  - **`Ctrl+U`** instantly clears the address bar to enter a new website.
  - **`Ctrl+W`** deletes the preceding word / path segment.
  - Multi-byte UTF-8 character editing, `Home` / `End`, `Backspace`, and `Delete`.
  - Automatic horizontal scrolling keeps the cursor in view when URLs exceed screen width.

- 🌐 **Smart URL Resolution**
  - Domains without schemes (e.g. `google.com`, `crates.io`) automatically default to `https://`.
  - In-page relative paths (`/about`, `page.html`, `//cdn`) and subdomains (`sub.domain.com`) resolve properly.
  - Root paths (typing `/` or `/docs` in the URL bar) navigate within the current website.

- 🛡️ **Crash-Resilient & Terminal-Safe**
  - **Global Panic Hook**: Always restores terminal raw mode and alternate screens if an unexpected crash occurs, ensuring your shell is never left broken or unreadable.
  - **Parser Isolation**: HTML parsing is wrapped in panic boundaries (`catch_unwind`) to catch malformed HTML without exiting the app.
  - **Network Timeouts**: Connect and request timeouts guard against hanging connections.

---

## ⌨️ Complete Keybindings Cheat Sheet

### 🔗 Navigation & Links
| Shortcut | Action |
| :--- | :--- |
| `Enter` | Navigate to address bar URL, or follow highlighted link |
| `Tab` | Select next link on page (auto-scrolls viewport) |
| `Shift + Tab` | Select previous link on page (auto-scrolls viewport) |
| `Ctrl + L` or `F2` | Open **Jump to Link #** popup modal |
| `Esc` | Cancel loading / close popup / deselect link / dismiss error |

### ✏️ Address Bar & Editing
| Shortcut | Action |
| :--- | :--- |
| `←` / `→` | Move cursor backward / forward by 1 character |
| `Home` / `Ctrl + A` | Move cursor to beginning of address bar |
| `End` / `Ctrl + E` | Move cursor to end of address bar |
| `Ctrl + U` | Clear the entire address bar |
| `Ctrl + W` | Delete previous word / path segment |
| `Backspace` | Delete character before cursor |
| `Delete` | Delete character under cursor |

### 📜 Page Scrolling & System
| Shortcut | Action |
| :--- | :--- |
| `↑` / `↓` | Scroll up / down by 1 line |
| `PgUp` / `PgDn` | Scroll up / down by 1 page |
| `Home` / `End` | Jump to top / bottom of page (when a link is selected) |
| `Ctrl + Q` | Quit RIFT-CLI |

---

## 📁 Project Architecture

```
rift-cli/
├── src/
│   ├── main.rs      # Event loop, state machine, URL resolution, popup & drawing
│   ├── html.rs      # HTML parser extracting headings, links, paragraphs & code
│   ├── renderer.rs  # Ratatui text layout, line-wrapping & link auto-scrolling
│   ├── network.rs   # Asynchronous HTTP client with timeouts & friendly errors
│   └── input.rs     # Cross-platform keyboard event listener (Crossterm)
├── Cargo.toml       # Dependencies and package metadata
└── README.md
```

---

## 🤝 How to Contribute

Contributions, bug reports, and suggestions are warmly welcome!

### Submitting a Pull Request (PR)

1. **Fork** the repository on GitHub:
   [https://github.com/Valtriaz/rift-cli/fork](https://github.com/Valtriaz/rift-cli/fork)

2. **Clone** your fork and create a new branch:
   ```bash
   git clone https://github.com/<your-username>/rift-cli.git
   cd rift-cli
   git checkout -b your-feature-name
   ```

3. **Make your changes** and verify the test suite:
   ```bash
   cargo test
   ```

4. **Commit** your changes using conventional commit messages:
   ```bash
   git add .
   git commit -m "add support for bookmarking pages"
   ```

5. **Push** to your branch and open a Pull Request:
   ```bash
   git push origin your-feature-name
   ```

---

## ☕ Support the Project

If you find **RIFT-CLI** useful, consider supporting its development:

<p align="center">
  <a href="https://buymeacoffee.com/valtriaz">
    <img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me A Coffee">
  </a>
</p>

- ☕ **[Buy Me a Coffee](https://buymeacoffee.com/valtriaz)** to sponsor ongoing work and new features.
- ⭐ **Star the repository** on GitHub to help others discover the project!
- 🐛 **Report issues & suggest ideas** in [GitHub Issues](https://github.com/Valtriaz/rift-cli/issues).

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).
