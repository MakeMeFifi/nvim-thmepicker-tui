use crate::themes::{
    detect_colorscheme_in_plugin_dir, fetch_dotfyle_page, load_initial_themes, ThemeEntry,
};
use anyhow::{Context, Result};
use std::collections::{BTreeMap, HashSet};
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
            FilterTab::All => "1: All Themes",
            FilterTab::Installed => "2: Installed",
            FilterTab::DotfyleTop => "3: Dotfyle Top",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewLanguage {
    AvrC,
    Rust,
    Python,
    Cpp,
}

impl PreviewLanguage {
    pub fn next(self) -> Self {
        match self {
            PreviewLanguage::AvrC => PreviewLanguage::Rust,
            PreviewLanguage::Rust => PreviewLanguage::Python,
            PreviewLanguage::Python => PreviewLanguage::Cpp,
            PreviewLanguage::Cpp => PreviewLanguage::AvrC,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PreviewLanguage::AvrC => "Embedded AVR C",
            PreviewLanguage::Rust => "Rust",
            PreviewLanguage::Python => "Python",
            PreviewLanguage::Cpp => "C++20",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListRow {
    Theme(usize),
    LoadMoreButton,
}

pub struct App {
    pub themes: Vec<ThemeEntry>,
    pub rows: Vec<ListRow>,
    pub selected_pos: usize,
    pub visible_limit: usize,
    pub last_fetched_page: usize,
    pub has_more_remote_pages: bool,
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

        let (themes, fetched_page) = load_initial_themes(&lazy_root);
        let active_colorscheme = detect_active_colorscheme(&config_file_path)
            .unwrap_or_else(|| "tokyonight".to_string());

        let mut app = Self {
            themes,
            rows: Vec::new(),
            selected_pos: 0,
            visible_limit: 10,
            last_fetched_page: fetched_page,
            has_more_remote_pages: true,
            active_colorscheme,
            tab: FilterTab::All,
            preview_lang: PreviewLanguage::AvrC,
            search_query: String::new(),
            is_searching: false,
            show_help: false,
            status_message: "Ready — Showing first 10 themes. Select '[ + Load More (+10) ]' or press [m] to fetch more from Dotfyle.".to_string(),
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
        let matching_indices: Vec<usize> = self
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

        let total_matching = matching_indices.len();
        let take_count = if self.tab == FilterTab::Installed || !query.is_empty() {
            total_matching
        } else {
            self.visible_limit.min(total_matching)
        };

        self.rows = matching_indices
            .into_iter()
            .take(take_count)
            .map(ListRow::Theme)
            .collect();

        // Append the interactive "[ + Load More (+10) ]" button row when in All or DotfyleTop tab
        let can_show_more = self.tab != FilterTab::Installed
            && query.is_empty()
            && (total_matching > take_count || self.has_more_remote_pages);

        if can_show_more {
            self.rows.push(ListRow::LoadMoreButton);
        }

        if self.selected_pos >= self.rows.len() {
            self.selected_pos = self.rows.len().saturating_sub(1);
        }
    }

    fn select_active_theme_if_present(&mut self) {
        for (pos, row) in self.rows.iter().enumerate() {
            if let ListRow::Theme(idx) = row {
                let cs = &self.themes[*idx].colorscheme;
                if cs == &self.active_colorscheme
                    || (self.active_colorscheme == "tokyonight" && cs == "tokyonight-night")
                {
                    self.selected_pos = pos;
                    break;
                }
            }
        }
    }

    pub fn current_row(&self) -> Option<ListRow> {
        self.rows.get(self.selected_pos).copied()
    }

    /// Returns the currently highlighted ThemeEntry (or the last visible theme if cursor is on Load More)
    pub fn current_theme(&self) -> Option<&ThemeEntry> {
        match self.current_row()? {
            ListRow::Theme(idx) => self.themes.get(idx),
            ListRow::LoadMoreButton => {
                for row in self.rows.iter().rev() {
                    if let ListRow::Theme(idx) = row {
                        return self.themes.get(*idx);
                    }
                }
                None
            }
        }
    }

    pub fn move_up(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        if self.selected_pos == 0 {
            self.selected_pos = self.rows.len() - 1;
        } else {
            self.selected_pos -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.rows.is_empty() {
            return;
        }
        self.selected_pos = (self.selected_pos + 1) % self.rows.len();
    }

    /// Load 10 more themes (fetching the next page live from Dotfyle if needed)
    pub fn load_more_themes(&mut self) {
        let next_limit = self.visible_limit + 10;
        let current_dotfyle_count = self
            .themes
            .iter()
            .filter(|t| t.dotfyle_rank.is_some())
            .count();

        if next_limit > current_dotfyle_count && self.has_more_remote_pages {
            let next_page = self.last_fetched_page + 1;
            let next_rank_start = current_dotfyle_count + 1;
            let fetched = fetch_dotfyle_page(next_page, next_rank_start, &self.lazy_root);

            if fetched.is_empty() {
                self.has_more_remote_pages = false;
            } else {
                self.last_fetched_page = next_page;
                let mut existing_repos: HashSet<String> =
                    self.themes.iter().map(|t| t.repo.to_lowercase()).collect();

                // Insert newly fetched Dotfyle themes right after existing Dotfyle themes
                let mut insert_pos = self
                    .themes
                    .iter()
                    .rposition(|t| t.dotfyle_rank.is_some())
                    .map(|p| p + 1)
                    .unwrap_or(self.themes.len());

                for item in fetched {
                    if existing_repos.insert(item.repo.to_lowercase()) {
                        self.themes.insert(insert_pos, item);
                        insert_pos += 1;
                    }
                }
            }
        }

        self.visible_limit = next_limit;
        self.recompute_filter();

        let shown_themes = self
            .rows
            .iter()
            .filter(|r| matches!(r, ListRow::Theme(_)))
            .count();
        self.status_is_error = false;
        self.status_message = format!(
            "Loaded +10 themes from dotfyle.com/neovim/colorscheme/top (Now showing {} themes).",
            shown_themes
        );
    }

    pub fn refresh_installed_states(&mut self) {
        for entry in &mut self.themes {
            if entry.is_builtin {
                entry.installed = true;
            } else {
                let dir = self.lazy_root.join(&entry.lazy_dir);
                entry.installed = dir.is_dir();
                if entry.installed {
                    entry.colorscheme = detect_colorscheme_in_plugin_dir(&dir, &entry.colorscheme);
                }
            }
        }
        self.recompute_filter();
    }

    /// Safely installs (if needed) and activates the currently selected colorscheme
    pub fn apply_selected_theme(&mut self) -> Result<()> {
        let Some(ListRow::Theme(idx)) = self.current_row() else {
            return Ok(());
        };
        let mut selected = self.themes[idx].clone();

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
                    .with_context(|| format!("Failed to execute git clone for {}", selected.repo))?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    self.status_is_error = true;
                    self.status_message = format!(
                        "Download failed for {} (Neovim config untouched): {}",
                        selected.repo,
                        stderr.lines().next().unwrap_or("Unknown git error")
                    );
                    return Ok(());
                }
            }
        }

        // Verify exact colorscheme file inside plugin's colors/ directory after clone
        let plugin_dir = self.lazy_root.join(&selected.lazy_dir);
        if plugin_dir.is_dir() {
            selected.colorscheme =
                detect_colorscheme_in_plugin_dir(&plugin_dir, &selected.colorscheme);
            self.themes[idx].colorscheme = selected.colorscheme.clone();
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
            .with_context(|| format!("Could not write {:?}", self.config_file_path))?;

        self.active_colorscheme = selected.colorscheme.clone();
        self.status_is_error = false;
        self.status_message = format!(
            "✓ Activated '{}' (:colorscheme {}) in ~/.config/nvim/lua/plugins/colorscheme.lua",
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
                "↺ Restored previous backup (Active theme: '{}').",
                self.active_colorscheme
            );
        } else if self.config_file_path.is_file() {
            fs::remove_file(&self.config_file_path)?;
            self.active_colorscheme = "tokyonight".to_string();
            self.status_is_error = false;
            self.status_message = "↺ Reset to LazyVim default theme ('tokyonight').".to_string();
        } else {
            self.status_is_error = false;
            self.status_message = "Already using LazyVim default theme ('tokyonight').".to_string();
        }
        Ok(())
    }

    fn build_safe_colorscheme_lua(&self, selected: &ThemeEntry) -> String {
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
