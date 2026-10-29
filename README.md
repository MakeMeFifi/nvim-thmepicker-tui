# 🎨 nvim-theme-tui — LazyVim & Neovim Colorscheme Studio

[![Crates.io](https://img.shields.io/crates/v/nvim-theme-tui.svg?logo=rust)](https://crates.io/crates/nvim-theme-tui)
[![Downloads](https://img.shields.io/crates/d/nvim-theme-tui.svg)](https://crates.io/crates/nvim-theme-tui)
[![Rust](https://img.shields.io/badge/Language-Rust_2021-orange?logo=rust)](https://www.rust-lang.org/)
[![UI](https://img.shields.io/badge/TUI-Ratatui_0.29-blueviolet)](https://ratatui.rs/)
[![Neovim](https://img.shields.io/badge/Editor-Neovim_%2F_LazyVim-57A143?logo=neovim)](https://www.lazyvim.org/)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)

A fast, clean Terminal User Interface (**TUI**) written in **Rust** (powered by [`ratatui`](https://ratatui.rs/) & [`crossterm`](https://github.com/crossterm-rs/crossterm)) for browsing, live-previewing, and **safely switching** local **LazyVim / Neovim** colorschemes alongside live-fetched themes from **[dotfyle.com/neovim/colorscheme/top](https://dotfyle.com/neovim/colorscheme/top)**.

---

## 📦 Installation Tutorial (via Cargo)

`nvim-theme-tui` is published on **[crates.io/crates/nvim-theme-tui](https://crates.io/crates/nvim-theme-tui)** and can be installed on any system with Rust & Cargo.

### Step 1: Install Rust & Cargo (if not already installed)
```bash
# Via rustup (Universal):
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Or on Fedora Linux:
sudo dnf install rust cargo git curl
```

### Step 2: Install `nvim-theme-tui` from Crates.io
```bash
cargo install nvim-theme-tui
```

### Step 3: Ensure `~/.cargo/bin` is in your `PATH`
Cargo installs binaries into `~/.cargo/bin`. If `nvim-theme-tui` is not found immediately, add this line to your `~/.bashrc` or `~/.zshrc`:
```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Step 4: Launch the TUI
```bash
nvim-theme-tui
```

### Upgrading to the Latest Version
To update an existing installation to the newest release from Crates.io:
```bash
cargo install nvim-theme-tui --force
```

---

## ✨ Features

- **🔎 Live Dotfyle Search & Fetch (`/ <query> + Enter`):**
  - Search for **any** Neovim colorscheme on **[dotfyle.com](https://dotfyle.com/neovim/colorscheme/top)** directly from the TUI.
  - Press **`/`**, type a theme name or keyword (e.g., `monokai`, `solarized`, `ayu`, `poimandres`), and press **`Enter`** (or select `[ 🌐 Search "<query>" on Dotfyle.com ]`) to fetch matching colorschemes live from `https://dotfyle.com/neovim/colorscheme/top?q=<query>`.
  - Fetched search results immediately get a 24-bit TrueColor live code preview and can be installed + activated in LazyVim with a single **`Enter`** press.
- **🌐 Live Dotfyle Fetching + "+10 Load More" Pagination:**
  - Fetches top Neovim colorschemes directly from **[dotfyle.com/neovim/colorscheme/top](https://dotfyle.com/neovim/colorscheme/top)** (with automatic offline fallback).
  - Displays the **first 10 themes** initially to keep the interface clean and focused.
  - Select **`[ + Load More (+10 from Dotfyle) ]`** at the bottom of the list (or press **`m`**) to dynamically load 10 additional themes per click across Dotfyle pages.
- **🔍 Local LazyVim Plugin Scanner:**
  - Automatically scans `~/.local/share/nvim/lazy/` to detect all locally installed theme plugins and marks your currently active colorscheme (`● ACTIVE`).
- **🌈 Live 24-Bit TrueColor Code Preview:**
  - Renders a real-time syntax-highlighted code preview right next to the theme list using the 24-bit RGB palette of the highlighted theme, including active cursorline, line numbers, simulated `lualine` statusline, and color swatches.
  - Press **`l`** to switch the live preview snippet between **Embedded AVR C**, **Rust**, **Python**, and **C++20**.
- **🛡️ Zero-Breakage Guarantee for Neovim:**
  - Writes exclusively to `~/.config/nvim/lua/plugins/colorscheme.lua`.
  - Automatically creates a backup (`~/.config/nvim/lua/plugins/colorscheme.lua.bak`) before every change (restorable anytime with **`r`**).
  - Pre-clones missing Dotfyle themes via `git clone --depth 1` into `~/.local/share/nvim/lazy/` and verifies the `colors/` directory before updating your config.
  - Wraps `vim.cmd.colorscheme` in a safe `pcall()` guard with automatic fallback to `tokyonight`.

---

## 🖥️ TUI Layout

```text
╭──────────────────────────────────────────╮╭───────────────────────────────────╮╭──────────────────────╮
│ nvim-theme-tui │ Active:  tokyonight     ││  1: All Themes │ 2: Installed ... ││ / Press [/] to filter│
╰──────────────────────────────────────────╯╰───────────────────────────────────╯╰──────────────────────╯
╭─ Themes (Showing 10) ────────────────────╮╭─ Preview: Catppuccin Mocha (:colorscheme catppuccin-mocha)╮
│  ▸ #01 Catppuccin Mocha     ✓ Installed  ││  1 // Microchip Studio 7.0.2594 — ATmega644 ADC Driver    │
│    #02 TokyoNight Night     ● ACTIVE     ││  2 #include <avr/io.h>                                    │
│    #03 Kanagawa Wave        ★ 6.4k       ││  6 void adc_init(uint8_t channel) {                       │
│    #04 Rosé Pine            ★ 3.1k       ││  7     DDRA = 0xFF; // Configure Port A as R-2R output    │
│    #05 Nightfox             ★ 4.1k       ││  8     ADMUX = (1 << REFS0) | (channel & 0x07);           │
│    ...                                   ││  NORMAL   main  󰘚 ATmega644 [Debug|AVR] │ utf-8 │ 7:18   │
│        [ + Load More (+10 from Dotfyle) ]│╰───────────────────────────────────────────────────────────╯
╰──────────────────────────────────────────╯╭───────────────────────────────────────────────────────────╮
╭─ Details ────────────────────────────────╮│   ██ BG   ██ FG   ██ Keyw   ██ Func   ██ Str   ██ Type    │
│ Repo: catppuccin/nvim  •  Installed      │╰───────────────────────────────────────────────────────────╯
│ Soothing pastel theme for Neovim         │
╰──────────────────────────────────────────╯
 Ready — Showing first 10 themes. Select '[ + Load More (+10) ]' or press [m] to fetch more from Dotfyle.
 ↑/↓ Select   Enter Apply   m Load +10   Tab Filter   l Language   / Search   r Rollback   ? Help   q Quit
```

---

## ⌨️ Keybindings

| Key | Action |
| :--- | :--- |
| **`↑` / `↓`** or **`j` / `k`** | Navigate through themes (updates the live code preview immediately) |
| **`Enter`** or **`a`** | Safely download (if not yet installed) and activate the selected theme in LazyVim, or trigger **Load More (+10)** when the button is selected |
| **`m`** | **Load More (+10):** Fetch and display 10 more themes from Dotfyle |
| **`Tab`** or **`1` / `2` / `3`** | Switch filter tabs: `1: All Themes` / `2: Installed` / `3: Dotfyle Top` |
| **`l`** | Cycle preview programming language (*Embedded AVR C* → *Rust* → *Python* → *C++20*) |
| **`/`** | Start interactive search (filters locally while typing; press **`Enter`** to fetch matching themes live from **Dotfyle.com**) |
| **`r`** | Rollback: Restore previous backup (`colorscheme.lua.bak`) or LazyVim default |
| **`?`** | Toggle help modal |
| **`q`** or **`Esc`** | Clear active search (`Esc`) or quit application |

---

## 🛠️ Building from Source

```bash
git clone https://github.com/MakeMeFifi/nvim-thmepicker-tui.git
cd nvim-thmepicker-tui
cargo build --release

# Run interactive TUI:
./target/release/nvim-theme-tui

# Non-interactive CLI list (first 10 + Load More + Live Dotfyle Search):
./target/release/nvim-theme-tui --list
./target/release/nvim-theme-tui --list --load-more
./target/release/nvim-theme-tui --list --search monokai
```

---

## 📄 License

This project is licensed under the **GNU General Public License v3.0** — see the [LICENSE](LICENSE) file for details.
