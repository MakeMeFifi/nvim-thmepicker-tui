# 🎨 nvim-theme-tui — LazyVim & Neovim Colorscheme Studio

[![Rust](https://img.shields.io/badge/Language-Rust_2021-orange?logo=rust)](https://www.rust-lang.org/)
[![UI](https://img.shields.io/badge/TUI-Ratatui_0.29-blueviolet)](https://ratatui.rs/)
[![Neovim](https://img.shields.io/badge/Editor-Neovim_%2F_LazyVim-57A143?logo=neovim)](https://www.lazyvim.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Ein schnelles, deterministisches Terminal-User-Interface (**TUI**) in **Rust** (mit [`ratatui`](https://ratatui.rs/) & [`crossterm`](https://github.com/crossterm-rs/crossterm)), mit dem du lokal installierte **LazyVim / Neovim Themes** sowie die beliebtesten Themes von **[dotfyle.com/neovim/colorscheme/top](https://dotfyle.com/neovim/colorscheme/top)** live im Terminal betrachten, testen und **100 % ausfallsicher** aktivieren kannst.

---

## ✨ Features

- **🔍 Lokaler & Cloud-Katalog:**
  - Erkennt automatisch alle bereits in `~/.local/share/nvim/lazy/` installierten Themes (inkl. eigener lokaler Plugins).
  - Enthält die Top-Colorschemes von **[Dotfyle](https://dotfyle.com/neovim/colorscheme/top)** (u. a. *Catppuccin Mocha/Macchiato/Frappé/Latte*, *TokyoNight Night/Moon/Storm*, *Kanagawa Wave/Dragon*, *Rosé Pine*, *Gruvbox*, *Nightfox/Carbonfox/Nordfox*, *OneDark Pro*, *Everforest*, *Cyberdream*, *GitHub Dark*, *Dracula*).
- **🌈 Echte 24-Bit TrueColor Live-Code-Vorschau:**
  - Zeigt rechts neben der Auswahlliste sofort ein vollständig eingefärbtes Code-Snippet (Hintergrund, Keywords, Funktionen, Strings, Hardware-Register, Datentypen, Cursor-Zeile, Zeilennummern & simulierte `lualine`-Statusbar) sowie die 10-teilige Farbpalette als Swatches.
  - Unterstützt Live-Umschaltung der Vorschau-Programmiersprache (**Embedded AVR C**, **Rust**, **Python**, **C++20**) per Taste `l`.
- **🛡️ Zero-Breakage-Garantie für Neovim:**
  - Schreibt ausschließlich in `~/.config/nvim/lua/plugins/colorscheme.lua`.
  - Legt vor jeder Änderung automatisch ein Backup (`~/.config/nvim/lua/plugins/colorscheme.lua.bak`) an (jederzeit per Taste `r` wiederherstellbar).
  - Lädt noch nicht installierte Dotfyle-Themes **vor** dem Umschalten per `git clone --depth 1` nach `~/.local/share/nvim/lazy/` herunter.
  - Schützt den Neovim-Start durch einen `pcall(vim.cmd.colorscheme, ...)`-Wrapper mit automatischem Fallback auf `tokyonight`, sodass Neovim selbst im Fehlerfall niemals mit einer Fehlermeldung startet.

---

## 🖥️ TUI-Layout

```text
┌─ Neovim / LazyVim ─────────────────────────┐┌─ Filter [Tab / 1-3] ──────────────┐┌─ Suche [/] ──────────┐
│ 🎨 LazyVim Theme Studio │ Aktiv: catppuccin ││ 1: Alle │ 2: Installiert │ 3: Top ││ 🔎 Drücke [/]...     │
└────────────────────────────────────────────┘└───────────────────────────────────┘└──────────────────────┘
┌─ Themes (21/21) — [↑/↓] Auswahl ───────────┐┌─ Live Code-Vorschau: Catppuccin Mocha │ Sprache [l]: AVR ─┐
│ ▶ ★ AKTIV ☾ Catppuccin Mocha   [Dotfyle #1]││  1 // Microchip Studio 7.0.2594 — ATmega644 AD/DA Wandler │
│   ✓ Inst. ☾ Catppuccin Macch.  [Dotfyle #1]││  2 #include <avr/io.h>                                    │
│   ✓ Inst. ☾ TokyoNight Night   [Dotfyle #2]││  6 void adc_init(uint8_t channel) {                       │
│   ↓ Cloud ☾ Kanagawa Wave      [Dotfyle #3]││  7     DDRA = 0xFF; // Port A als Ausgang                 │
│   ↓ Cloud ☾ Rosé Pine          [Dotfyle #4]││  8     ADMUX = (1 << REFS0) | (channel & 0x07);           │
│   ↓ Cloud ☾ Gruvbox Dark       [Dotfyle #5]││  NORMAL   main  󰘚 ATmega644 [Debug|AVR] │ utf-8 │ 7:18   │
├────────────────────────────────────────────┤├───────────────────────────────────────────────────────────┤
│ Colorscheme: catppuccin-mocha              ││  ███   ███   ███   ███   ███   ███   ███   ███   ███  ███ │
│ Status: Lokal installiert                  ││  BG    FG    Keyw  Func  Str   Type  Macr  Num   Oper Com │
└────────────────────────────────────────────┘└───────────────────────────────────────────────────────────┘
```

---

## ⌨️ Tastenkürzel

| Taste | Aktion |
| :--- | :--- |
| **`↑` / `↓`** oder **`j` / `k`** | Durch die Themes navigieren (mit sofortiger Live-Vorschau) |
| **`Enter`** oder **`a`** | Ausgewähltes Theme sicher herunterladen (falls nötig) & in LazyVim aktivieren |
| **`Tab`** oder **`1` / `2` / `3`** | Filter wechseln: `1: Alle Themes` / `2: Installiert` / `3: Dotfyle Top` |
| **`l`** | Programmiersprache im Live-Code-Beispiel wechseln (*AVR C* → *Rust* → *Python* → *C++*) |
| **`/`** | Suchmodus starten (filtert nach Name, Repo oder Beschreibung) |
| **`r`** | Rollback: Letztes Backup (`colorscheme.lua.bak`) oder LazyVim-Standard wiederherstellen |
| **`?`** | Hilfe-Fenster ein-/ausblenden |
| **`q`** oder **`Esc`** | Programm beenden |

---

## 🚀 Installation & Ausführung

### Build aus dem Quellcode
```bash
cd /home/chris/Dokumente/Coding/rust/nvim-theme-tui
cargo build --release
```

### Starten
```bash
# Direkt über den verlinkten Befehl im Terminal:
nvim-theme-tui

# Oder über Cargo im Projektordner:
cargo run --release

# Nicht-interaktive Listenansicht im Terminal:
nvim-theme-tui --list
```

---

## 📂 Projektstruktur

```text
nvim-theme-tui/
├── Cargo.toml       # Abhängigkeiten (ratatui, crossterm, anyhow)
├── .gitignore       # Standard Rust & IDE Ignore-Regeln
├── README.md        # Projektdokumentation
└── src/
    ├── main.rs      # Terminal-Initialisierung, CLI-Argumente & Event-Loop
    ├── app.rs       # State-Management, Suche, git-clone & sicherer Lua-Config-Generator
    ├── themes.rs    # Kuratierter Dotfyle-Top-Katalog + 24-Bit-Paletten + lokaler LazyVim-Scanner
    └── ui.rs        # Ratatui Split-Screen-Rendering, Code-Highlighter & Farb-Swatches
```
