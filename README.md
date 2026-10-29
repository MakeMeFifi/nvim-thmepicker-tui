# 🎨 nvim-theme-tui — LazyVim & Neovim Colorscheme Studio

[![Rust](https://img.shields.io/badge/Language-Rust_2021-orange?logo=rust)](https://www.rust-lang.org/)
[![UI](https://img.shields.io/badge/TUI-Ratatui_0.29-blueviolet)](https://ratatui.rs/)
[![Neovim](https://img.shields.io/badge/Editor-Neovim_%2F_LazyVim-57A143?logo=neovim)](https://www.lazyvim.org/)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)

A fast, deterministic Terminal User Interface (**TUI**) written in **Rust** (powered by [`ratatui`](https://ratatui.rs/) & [`crossterm`](https://github.com/crossterm-rs/crossterm)) for browsing, live-previewing, and **safely switching** local **LazyVim / Neovim** colorschemes alongside the most popular community themes from **[dotfyle.com/neovim/colorscheme/top](https://dotfyle.com/neovim/colorscheme/top)**.

---

## ✨ Features

- **🔍 Unified Local & Cloud Catalog:**
  - Automatically scans `~/.local/share/nvim/lazy/` to discover all currently installed Neovim theme plugins.
  - Ships with a curated catalog of the top Neovim colorschemes from **[Dotfyle](https://dotfyle.com/neovim/colorscheme/top)** (including *Catppuccin Mocha/Macchiato/Frappé/Latte*, *TokyoNight Night/Moon/Storm*, *Kanagawa Wave/Dragon*, *Rosé Pine*, *Gruvbox*, *Nightfox/Carbonfox/Nordfox*, *OneDark Pro*, *Everforest*, *Cyberdream*, *GitHub Dark*, and *Dracula*).
- **🌈 Live 24-Bit TrueColor Code Preview:**
  - Renders a real-time syntax-highlighted code snippet right next to the theme selector using the exact 24-bit RGB palette of the highlighted theme (background, keywords, functions, strings, hardware registers/macros, types, active cursorline, line numbers, and a simulated `lualine` status bar) plus 10 color swatches.
  - Press `l` to cycle the live preview snippet across **Embedded AVR C**, **Rust**, **Python**, and **C++20**.
- **🛡️ Zero-Breakage Guarantee for Neovim:**
  - Writes exclusively to an isolated plugin file: `~/.config/nvim/lua/plugins/colorscheme.lua`.
  - Automatically creates a backup (`~/.config/nvim/lua/plugins/colorscheme.lua.bak`) before every change, which can be restored at any time by pressing `r`.
  - Pre-clones missing Dotfyle themes via `git clone --depth 1` into `~/.local/share/nvim/lazy/` **before** updating the configuration.
  - Wraps the colorscheme activation in a `pcall(vim.cmd.colorscheme, ...)` guard with an automatic fallback to `tokyonight`, ensuring Neovim never starts with an error.

---

## 🖥️ TUI Layout

```text
┌─ Neovim / LazyVim ─────────────────────────┐┌─ Filter [Tab / 1-3] ──────────────┐┌─ Search [/] ─────────┐
│ 🎨 LazyVim Theme Studio │ Active: catppucc ││ 1: All  │ 2: Installed │ 3: Top   ││ 🔎 Press [/]...      │
└────────────────────────────────────────────┘└───────────────────────────────────┘└──────────────────────┘
┌─ Themes (21/21) — [↑/↓] Select ────────────┐┌─ Live Code Preview: Catppuccin Mocha │ Lang [l]: AVR C ───┐
│ ▶ ★ ACTIVE ☾ Catppuccin Mocha  [Dotfyle #1]││  1 // Microchip Studio 7.0.2594 — ATmega644 AD/DA         │
│   ✓ Inst.  ☾ Catppuccin Macch. [Dotfyle #1]││  2 #include <avr/io.h>                                    │
│   ✓ Inst.  ☾ TokyoNight Night  [Dotfyle #2]││  6 void adc_init(uint8_t channel) {                       │
│   ↓ Cloud  ☾ Kanagawa Wave     [Dotfyle #3]││  7     DDRA = 0xFF; // Configure Port A as output         │
│   ↓ Cloud  ☾ Rosé Pine         [Dotfyle #4]││  8     ADMUX = (1 << REFS0) | (channel & 0x07);           │
│   ↓ Cloud  ☾ Gruvbox Dark      [Dotfyle #5]││  NORMAL   main  󰘚 ATmega644 [Debug|AVR] │ utf-8 │ 7:18   │
├────────────────────────────────────────────┤├───────────────────────────────────────────────────────────┤
│ Colorscheme: catppuccin-mocha              ││  ███   ███   ███   ███   ███   ███   ███   ███   ███  ███ │
│ Status: Installed locally                  ││  BG    FG    Keyw  Func  Str   Type  Macr  Num   Oper Com │
└────────────────────────────────────────────┘└───────────────────────────────────────────────────────────┘
```

---

## ⌨️ Keybindings

| Key | Action |
| :--- | :--- |
| **`↑` / `↓`** or **`j` / `k`** | Navigate through themes (updates the live code preview immediately) |
| **`Enter`** or **`a`** | Safely download (if not yet installed) and activate the selected theme in LazyVim |
| **`Tab`** or **`1` / `2` / `3`** | Switch filter tabs: `1: All Themes` / `2: Installed` / `3: Dotfyle Top` |
| **`l`** | Cycle preview programming language (*Embedded AVR C* → *Rust* → *Python* → *C++20*) |
| **`/`** | Start interactive search (filters by name, repository, or description) |
| **`r`** | Rollback: Restore previous backup (`colorscheme.lua.bak`) or LazyVim default |
| **`?`** | Toggle help modal |
| **`q`** or **`Esc`** | Quit application |

---

## 🚀 Installation & Usage

### Install via Cargo (from Crates.io or Git)
```bash
# Install directly from GitHub:
cargo install --git https://github.com/MakeMeFifi/nvim-thmepicker-tui.git

# Or install from Crates.io (once published):
cargo install nvim-theme-tui
```

### Build from Source
```bash
git clone https://github.com/MakeMeFifi/nvim-thmepicker-tui.git
cd nvim-thmepicker-tui
cargo build --release
```

### Run
```bash
# Launch interactive TUI:
nvim-theme-tui

# Non-interactive CLI list of all detected & Dotfyle themes:
nvim-theme-tui --list
```

---

## 📂 Project Structure

```text
nvim-theme-tui/
├── Cargo.toml       # Package manifest & dependencies (ratatui, crossterm, anyhow)
├── LICENSE          # GPL-3.0 License
├── .gitignore       # Rust & IDE ignore rules
├── README.md        # Documentation
└── src/
    ├── main.rs      # Terminal initialization, CLI arguments & event loop
    ├── app.rs       # State management, search, git-clone installer & safe Lua config writer
    ├── themes.rs    # Curated Dotfyle Top catalog + 24-bit RGB palettes + local LazyVim scanner
    └── ui.rs        # Ratatui split-screen renderer, syntax highlighter & palette swatches
```

---

## 📄 License

This project is licensed under the **GNU General Public License v3.0** — see the [LICENSE](LICENSE) file for details.
