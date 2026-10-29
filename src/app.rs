use crate::themes::{load_all_themes, ThemeEntry};
use anyhow::{Context, Result};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterTab {
    All,
    Installed,
    DotfyleTop,
}

impl FilterTab {
    pub fn next(self) -> Self {
        match self {
            FilterTab::All => FilterTab::Installed,
            FilterTab::Installed => FilterTab::DotfyleTop,
            FilterTab::DotfyleTop => FilterTab::All,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            FilterTab::All => "1: Alle Themes",
            FilterTab::Installed => "2: Installiert",
            FilterTab::DotfyleTop => "3: Dotfyle Top",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewLanguage {
    Rust,
    AvrC,
    Python,
    Cpp,
}

impl PreviewLanguage {
    pub fn next(self) -> Self {
        match self {
            PreviewLanguage::Rust => PreviewLanguage::AvrC,
            PreviewLanguage::AvrC => PreviewLanguage::Python,
            PreviewLanguage::Python => PreviewLanguage::Cpp,
            PreviewLanguage::Cpp => PreviewLanguage::Rust,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PreviewLanguage::Rust => "Rust (Cargo)",
            PreviewLanguage::AvrC => "Embedded AVR C (Microchip)",
            PreviewLanguage::Python => "Python (uv / Pyright)",
            PreviewLanguage::Cpp => "C++20 (clangd / CMake)",
        }
    }
}

pub struct App {
    pub themes: Vec<ThemeEntry>,
    pub filtered_indices: Vec<usize>,
    pub selected_pos: usize,
    pub active_colorscheme: String,
    pub tab: FilterTab,
    pub preview_lang: PreviewLanguage,
    pub search_query: String,
    pub is_searching: bool,
    pub show_help: bool,
    pub status_message: String,
    pub status_is_error: bool,
    pub config_file_path: PathBuf,
    pub backup_file_path: PathBuf,
    pub lazy_root: PathBuf,
}

impl App {
    pub fn new() -> Self {
        let home = env::var("HOME").unwrap_or_else(|_| "/home/chris".to_string());
        let home_path = PathBuf::from(home);
        let lazy_root = home_path.join(".local/share/nvim/lazy");
        let config_file_path = home_path.join(".config/nvim/lua/plugins/colorscheme.lua");
        let backup_file_path = home_path.join(".config/nvim/lua/plugins/colorscheme.lua.bak");

        let themes = load_all_themes(&lazy_root);
        let active_colorscheme = detect_active_colorscheme(&config_file_path)
            .unwrap_or_else(|| "tokyonight".to_string());

        let mut app = Self {
            themes,
            filtered_indices: Vec::new(),
            selected_pos: 0,
            active_colorscheme,
            tab: FilterTab::All,
            preview_lang: PreviewLanguage::AvrC,
            search_query: String::new(),
            is_searching: false,
            show_help: false,
            status_message: "Bereit — [↑/↓ oder j/k] Auswahl | [Enter] Theme sicher aktivieren | [Tab] Filter | [l] Code-Sprache | [q] Beenden".to_string(),
            status_is_error: false,
            config_file_path,
            backup_file_path,
            lazy_root,
        };
        app.recompute_filter();
        app.select_active_theme_if_present();
        app
    }

    pub fn recompute_filter(&mut self) {
        let query = self.search_query.trim().to_lowercase();
        self.filtered_indices = self
            .themes
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                let tab_ok = match self.tab {
                    FilterTab::All => true,
                    FilterTab::Installed => t.installed,
                    FilterTab::DotfyleTop => t.dotfyle_rank.is_some(),
                };
                if !tab_ok {
                    return false;
                }
                if query.is_empty() {
                    return true;
                }
                t.display_name.to_lowercase().contains(&query)
                    || t.colorscheme.to_lowercase().contains(&query)
                    || t.repo.to_lowercase().contains(&query)
                    || t.description.to_lowercase().contains(&query)
            })
            .map(|(i, _)| i)
            .collect();

        if self.selected_pos >= self.filtered_indices.len() {
            self.selected_pos = self.filtered_indices.len().saturating_sub(1);
        }
    }

    fn select_active_theme_if_present(&mut self) {
        for (pos, &idx) in self.filtered_indices.iter().enumerate() {
            let cs = &self.themes[idx].colorscheme;
            if cs == &self.active_colorscheme
                || (self.active_colorscheme == "tokyonight" && cs == "tokyonight-moon")
            {
                self.selected_pos = pos;
                break;
            }
        }
    }

    pub fn current_theme(&self) -> Option<&ThemeEntry> {
        let &idx = self.filtered_indices.get(self.selected_pos)?;
        self.themes.get(idx)
    }

    pub fn move_up(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        if self.selected_pos == 0 {
            self.selected_pos = self.filtered_indices.len() - 1;
        } else {
            self.selected_pos -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.filtered_indices.is_empty() {
            return;
        }
        self.selected_pos = (self.selected_pos + 1) % self.filtered_indices.len();
    }

    pub fn refresh_installed_states(&mut self) {
        for entry in &mut self.themes {
            if entry.is_builtin {
                entry.installed = true;
            } else {
                entry.installed = self.lazy_root.join(&entry.lazy_dir).is_dir();
            }
        }
        self.recompute_filter();
    }

    /// Safely installs (if needed) and activates the currently selected colorscheme
    /// by writing ONLY to ~/.config/nvim/lua/plugins/colorscheme.lua with automatic pcall fallback.
    pub fn apply_selected_theme(&mut self) -> Result<()> {
        let Some(&idx) = self.filtered_indices.get(self.selected_pos) else {
            return Ok(());
        };
        let selected = self.themes[idx].clone();

        // 1. If the plugin is not installed yet in ~/.local/share/nvim/lazy, clone it first!
        if !selected.is_builtin && !selected.installed {
            let target_dir = self.lazy_root.join(&selected.lazy_dir);
            if !target_dir.exists() {
                let clone_url = format!("https://github.com/{}.git", selected.repo);
                let output = Command::new("git")
                    .args([
                        "clone",
                        "--depth",
                        "1",
                        &clone_url,
                        target_dir.to_string_lossy().as_ref(),
                    ])
                    .output()
                    .with_context(|| format!("Fehler beim Ausführen von git clone für {}", selected.repo))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    self.status_is_error = true;
                    self.status_message = format!(
                        "Download von {} fehlgeschlagen (Neovim bleibt unverändert!): {}",
                        selected.repo,
                        stderr.lines().next().unwrap_or("Unbekannter Fehler")
                    );
                    return Ok(());
                }
            }
        }

        // 2. Refresh installed flags so the newly cloned plugin is included in colorscheme.lua
        self.refresh_installed_states();

        // 3. Create safety backup of existing colorscheme.lua if present
        if let Some(parent) = self.config_file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        if self.config_file_path.is_file() {
            let _ = fs::copy(&self.config_file_path, &self.backup_file_path);
        }

        // 4. Generate safe Lua configuration with pcall fallback so Neovim can NEVER break
        let lua_content = self.build_safe_colorscheme_lua(&selected);
        fs::write(&self.config_file_path, lua_content)
            .with_context(|| format!("Konnte {:?} nicht schreiben", self.config_file_path))?;

        self.active_colorscheme = selected.colorscheme.clone();
        self.status_is_error = false;
        self.status_message = format!(
            "✓ Theme '{}' ({}) sicher aktiviert in ~/.config/nvim/lua/plugins/colorscheme.lua!",
            selected.display_name, selected.colorscheme
        );
        Ok(())
    }

    /// Rollback ~/.config/nvim/lua/plugins/colorscheme.lua to backup or default TokyoNight
    pub fn rollback_theme(&mut self) -> Result<()> {
        if self.backup_file_path.is_file() {
            fs::copy(&self.backup_file_path, &self.config_file_path)?;
            self.active_colorscheme = detect_active_colorscheme(&self.config_file_path)
                .unwrap_or_else(|| "tokyonight".to_string());
            self.status_is_error = false;
            self.status_message = format!(
                "↺ Backup wiederhergestellt (Aktives Theme: '{}').",
                self.active_colorscheme
            );
        } else if self.config_file_path.is_file() {
            fs::remove_file(&self.config_file_path)?;
            self.active_colorscheme = "tokyonight".to_string();
            self.status_is_error = false;
            self.status_message = "↺ Auf LazyVim Standard-Theme ('tokyonight') zurückgesetzt.".to_string();
        } else {
            self.status_is_error = false;
            self.status_message = "Bereits auf LazyVim Standard-Theme ('tokyonight').".to_string();
        }
        Ok(())
    }

    fn build_safe_colorscheme_lua(&self, selected: &ThemeEntry) -> String {
        // Keep all currently installed theme repos declared so Lazy.nvim never marks them for removal
        let mut repos: BTreeMap<String, Option<String>> = BTreeMap::new();
        for t in &self.themes {
            if !t.is_builtin && (t.installed || t.repo == selected.repo) && t.repo.contains('/') {
                repos.insert(t.repo.clone(), t.plugin_name_override.clone());
            }
        }

        let mut plugin_specs = String::new();
        for (repo, name_override) in repos {
            if let Some(name) = name_override {
                plugin_specs.push_str(&format!(
                    "  {{ \"{}\", name = \"{}\", lazy = false, priority = 1000 }},\n",
                    repo, name
                ));
            } else {
                plugin_specs.push_str(&format!(
                    "  {{ \"{}\", lazy = false, priority = 1000 }},\n",
                    repo
                ));
            }
        }

        format!(
            "-- ============================================================================\n\
             -- ~/.config/nvim/lua/plugins/colorscheme.lua\n\
             -- Managed safely by nvim-theme-tui (with automatic fallback protection)\n\
             -- Active Theme: {display_name} ({colorscheme})\n\
             -- ============================================================================\n\
             -- NVIM_THEME_TUI_ACTIVE = \"{colorscheme}\"\n\n\
             return {{\n\
             {plugin_specs}\
               {{\n\
                 \"LazyVim/LazyVim\",\n\
                 opts = {{\n\
                   colorscheme = function()\n\
                     local ok = pcall(vim.cmd.colorscheme, \"{colorscheme}\")\n\
                     if not ok then\n\
                       pcall(vim.cmd.colorscheme, \"tokyonight\")\n\
                     end\n\
                   end,\n\
                 }},\n\
               }},\n\
             }}\n",
            display_name = selected.display_name,
            colorscheme = selected.colorscheme,
            plugin_specs = plugin_specs,
        )
    }
}

fn detect_active_colorscheme(config_path: &PathBuf) -> Option<String> {
    let text = fs::read_to_string(config_path).ok()?;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("-- NVIM_THEME_TUI_ACTIVE = \"") {
            if let Some(val) = rest.strip_suffix('"') {
                return Some(val.to_string());
            }
        }
        if let Some(idx) = trimmed.find("colorscheme = \"") {
            let after = &trimmed[idx + "colorscheme = \"".len()..];
            if let Some(end) = after.find('"') {
                return Some(after[..end].to_string());
            }
        }
    }
    None
}
