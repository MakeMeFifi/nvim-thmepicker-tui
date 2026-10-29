use ratatui::style::Color;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// 24-bit TrueColor palette used to render the live code preview and color swatches.
#[derive(Debug, Clone, Copy)]
pub struct ThemePalette {
    pub bg: Color,
    pub bg_alt: Color,
    pub bg_highlight: Color,
    pub fg: Color,
    pub comment: Color,
    pub keyword: Color,
    pub func: Color,
    pub string: Color,
    pub number: Color,
    pub r#type: Color,
    pub macro_reg: Color,
    pub operator: Color,
    pub accent: Color,
}

pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

#[derive(Debug, Clone)]
pub struct ThemeEntry {
    pub display_name: String,
    pub colorscheme: String,
    pub repo: String,
    pub lazy_dir: String,
    pub plugin_name_override: Option<String>,
    pub dotfyle_rank: Option<usize>,
    pub is_light: bool,
    pub description: String,
    pub palette: ThemePalette,
    pub installed: bool,
    pub is_builtin: bool,
}

impl ThemeEntry {
    pub fn category_badge(&self) -> String {
        if let Some(rank) = self.dotfyle_rank {
            format!("Dotfyle #{}", rank)
        } else if self.is_builtin {
            "Built-in".to_string()
        } else {
            "Custom".to_string()
        }
    }
}

/// Returns the curated catalog of Top Neovim Colorschemes from https://dotfyle.com/neovim/colorscheme/top
/// plus built-in themes and dynamically discovered local LazyVim themes.
pub fn load_all_themes(lazy_root: &Path) -> Vec<ThemeEntry> {
    let mut entries = curated_dotfyle_themes();

    // Check which themes are already installed in ~/.local/share/nvim/lazy
    for entry in &mut entries {
        if entry.is_builtin {
            entry.installed = true;
        } else {
            entry.installed = lazy_root.join(&entry.lazy_dir).is_dir();
        }
    }

    // Also scan ~/.local/share/nvim/lazy/*/colors/*.{lua,vim} for any user-installed
    // themes not in the curated list so nothing installed is ever hidden.
    let known_schemes: HashSet<String> = entries
        .iter()
        .map(|e| e.colorscheme.to_lowercase())
        .collect();

    if let Ok(read_dir) = fs::read_dir(lazy_root) {
        for dir_entry in read_dir.flatten() {
            let plugin_path = dir_entry.path();
            if !plugin_path.is_dir() {
                continue;
            }
            let plugin_dir_name = dir_entry.file_name().to_string_lossy().to_string();
            let colors_dir = plugin_path.join("colors");
            if !colors_dir.is_dir() {
                continue;
            }

            let repo_guess = detect_git_remote_repo(&plugin_path).unwrap_or_else(|| plugin_dir_name.clone());

            if let Ok(color_files) = fs::read_dir(&colors_dir) {
                for cf in color_files.flatten() {
                    let path = cf.path();
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if ext != "lua" && ext != "vim" {
                        continue;
                    }
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if !known_schemes.contains(&stem.to_lowercase()) {
                            entries.push(ThemeEntry {
                                display_name: format!("{} (Installed)", stem),
                                colorscheme: stem.to_string(),
                                repo: repo_guess.clone(),
                                lazy_dir: plugin_dir_name.clone(),
                                plugin_name_override: None,
                                dotfyle_rank: None,
                                is_light: stem.contains("light") || stem.contains("day") || stem.contains("latte") || stem.contains("dawn"),
                                description: format!(
                                    "Locally installed colorscheme discovered in ~/.local/share/nvim/lazy/{}",
                                    plugin_dir_name
                                ),
                                palette: default_dark_palette(),
                                installed: true,
                                is_builtin: false,
                            });
                        }
                    }
                }
            }
        }
    }

    entries
}

fn detect_git_remote_repo(plugin_dir: &Path) -> Option<String> {
    let git_config = fs::read_to_string(plugin_dir.join(".git/config")).ok()?;
    for line in git_config.lines() {
        let trimmed = line.trim();
        if let Some(url) = trimmed.strip_prefix("url = ") {
            let cleaned = url
                .trim_end_matches(".git")
                .trim_start_matches("https://github.com/")
                .trim_start_matches("git@github.com:");
            if cleaned.contains('/') && !cleaned.contains(':') {
                return Some(cleaned.to_string());
            }
        }
    }
    None
}

const fn default_dark_palette() -> ThemePalette {
    ThemePalette {
        bg: rgb(26, 27, 38),
        bg_alt: rgb(22, 22, 30),
        bg_highlight: rgb(41, 46, 66),
        fg: rgb(192, 202, 245),
        comment: rgb(86, 95, 137),
        keyword: rgb(187, 154, 247),
        func: rgb(122, 162, 247),
        string: rgb(158, 206, 106),
        number: rgb(255, 158, 100),
        r#type: rgb(42, 195, 222),
        macro_reg: rgb(247, 118, 142),
        operator: rgb(137, 221, 255),
        accent: rgb(122, 162, 247),
    }
}

fn curated_dotfyle_themes() -> Vec<ThemeEntry> {
    vec![
        // 1. Catppuccin Mocha (Dotfyle #1)
        ThemeEntry {
            display_name: "Catppuccin Mocha".into(),
            colorscheme: "catppuccin-mocha".into(),
            repo: "catppuccin/nvim".into(),
            lazy_dir: "catppuccin".into(),
            plugin_name_override: Some("catppuccin".into()),
            dotfyle_rank: Some(1),
            is_light: false,
            description: "Soothing pastel theme for Neovim — warmest dark variant (Mocha). #1 on Dotfyle.".into(),
            palette: ThemePalette {
                bg: rgb(30, 30, 46),
                bg_alt: rgb(24, 24, 37),
                bg_highlight: rgb(49, 50, 68),
                fg: rgb(205, 214, 244),
                comment: rgb(108, 112, 134),
                keyword: rgb(203, 166, 247),
                func: rgb(137, 180, 250),
                string: rgb(166, 227, 161),
                number: rgb(250, 179, 135),
                r#type: rgb(249, 226, 175),
                macro_reg: rgb(243, 139, 168),
                operator: rgb(137, 220, 235),
                accent: rgb(203, 166, 247),
            },
            installed: false,
            is_builtin: false,
        },
        // 2. Catppuccin Macchiato
        ThemeEntry {
            display_name: "Catppuccin Macchiato".into(),
            colorscheme: "catppuccin-macchiato".into(),
            repo: "catppuccin/nvim".into(),
            lazy_dir: "catppuccin".into(),
            plugin_name_override: Some("catppuccin".into()),
            dotfyle_rank: Some(1),
            is_light: false,
            description: "Medium-dark soothing pastel theme with soft contrast (Macchiato).".into(),
            palette: ThemePalette {
                bg: rgb(36, 39, 58),
                bg_alt: rgb(30, 32, 48),
                bg_highlight: rgb(54, 58, 79),
                fg: rgb(202, 211, 245),
                comment: rgb(110, 115, 141),
                keyword: rgb(198, 160, 246),
                func: rgb(138, 173, 244),
                string: rgb(166, 218, 149),
                number: rgb(245, 169, 127),
                r#type: rgb(238, 212, 159),
                macro_reg: rgb(237, 135, 150),
                operator: rgb(145, 215, 227),
                accent: rgb(198, 160, 246),
            },
            installed: false,
            is_builtin: false,
        },
        // 3. Catppuccin Frappe
        ThemeEntry {
            display_name: "Catppuccin Frappé".into(),
            colorscheme: "catppuccin-frappe".into(),
            repo: "catppuccin/nvim".into(),
            lazy_dir: "catppuccin".into(),
            plugin_name_override: Some("catppuccin".into()),
            dotfyle_rank: Some(1),
            is_light: false,
            description: "Subdued slate-blue pastel dark variant (Frappé).".into(),
            palette: ThemePalette {
                bg: rgb(48, 52, 70),
                bg_alt: rgb(41, 44, 60),
                bg_highlight: rgb(65, 69, 89),
                fg: rgb(198, 208, 245),
                comment: rgb(115, 121, 148),
                keyword: rgb(202, 158, 230),
                func: rgb(140, 170, 238),
                string: rgb(166, 209, 137),
                number: rgb(239, 159, 118),
                r#type: rgb(229, 200, 144),
                macro_reg: rgb(231, 130, 132),
                operator: rgb(153, 209, 219),
                accent: rgb(202, 158, 230),
            },
            installed: false,
            is_builtin: false,
        },
        // 4. Catppuccin Latte (Light)
        ThemeEntry {
            display_name: "Catppuccin Latte (Light)".into(),
            colorscheme: "catppuccin-latte".into(),
            repo: "catppuccin/nvim".into(),
            lazy_dir: "catppuccin".into(),
            plugin_name_override: Some("catppuccin".into()),
            dotfyle_rank: Some(1),
            is_light: true,
            description: "Warm daylight pastel theme for bright environments (Latte).".into(),
            palette: ThemePalette {
                bg: rgb(239, 241, 245),
                bg_alt: rgb(230, 233, 239),
                bg_highlight: rgb(204, 208, 218),
                fg: rgb(76, 79, 105),
                comment: rgb(140, 143, 161),
                keyword: rgb(136, 57, 239),
                func: rgb(30, 102, 245),
                string: rgb(64, 160, 43),
                number: rgb(254, 100, 11),
                r#type: rgb(223, 142, 29),
                macro_reg: rgb(210, 15, 57),
                operator: rgb(4, 165, 229),
                accent: rgb(136, 57, 239),
            },
            installed: false,
            is_builtin: false,
        },
        // 5. TokyoNight Night (Dotfyle #2 — Default LazyVim Theme)
        ThemeEntry {
            display_name: "TokyoNight Night".into(),
            colorscheme: "tokyonight-night".into(),
            repo: "folke/tokyonight.nvim".into(),
            lazy_dir: "tokyonight.nvim".into(),
            plugin_name_override: None,
            dotfyle_rank: Some(2),
            is_light: false,
            description: "Clean dark theme celebrating the neon lights of downtown Tokyo at night.".into(),
            palette: default_dark_palette(),
            installed: false,
            is_builtin: false,
        },
        // 6. TokyoNight Moon
        ThemeEntry {
            display_name: "TokyoNight Moon".into(),
            colorscheme: "tokyonight-moon".into(),
            repo: "folke/tokyonight.nvim".into(),
            lazy_dir: "tokyonight.nvim".into(),
            plugin_name_override: None,
            dotfyle_rank: Some(2),
            is_light: false,
            description: "Soft indigo moonlight variant of TokyoNight (LazyVim default style).".into(),
            palette: ThemePalette {
                bg: rgb(34, 36, 54),
                bg_alt: rgb(30, 32, 48),
                bg_highlight: rgb(47, 51, 77),
                fg: rgb(200, 211, 245),
                comment: rgb(99, 109, 166),
                keyword: rgb(192, 153, 255),
                func: rgb(130, 170, 255),
                string: rgb(195, 232, 141),
                number: rgb(255, 150, 108),
                r#type: rgb(101, 188, 255),
                macro_reg: rgb(255, 117, 127),
                operator: rgb(137, 221, 255),
                accent: rgb(130, 170, 255),
            },
            installed: false,
            is_builtin: false,
        },
        // 7. TokyoNight Storm
        ThemeEntry {
            display_name: "TokyoNight Storm".into(),
            colorscheme: "tokyonight-storm".into(),
            repo: "folke/tokyonight.nvim".into(),
            lazy_dir: "tokyonight.nvim".into(),
            plugin_name_override: None,
            dotfyle_rank: Some(2),
            is_light: false,
            description: "Deep stormy slate blue variant of TokyoNight.".into(),
            palette: ThemePalette {
                bg: rgb(36, 40, 59),
                bg_alt: rgb(31, 35, 53),
                bg_highlight: rgb(41, 46, 66),
                fg: rgb(192, 202, 245),
                comment: rgb(86, 95, 137),
                keyword: rgb(187, 154, 247),
                func: rgb(122, 162, 247),
                string: rgb(158, 206, 106),
                number: rgb(255, 158, 100),
                r#type: rgb(42, 195, 222),
                macro_reg: rgb(247, 118, 142),
                operator: rgb(137, 221, 255),
                accent: rgb(122, 162, 247),
            },
            installed: false,
            is_builtin: false,
        },
        // 8. Kanagawa Wave (Dotfyle #3)
        ThemeEntry {
            display_name: "Kanagawa Wave".into(),
            colorscheme: "kanagawa-wave".into(),
            repo: "rebelot/kanagawa.nvim".into(),
            lazy_dir: "kanagawa.nvim".into(),
            plugin_name_override: None,
            dotfyle_rank: Some(3),
            is_light: false,
            description: "Dark colorscheme inspired by Katsushika Hokusai's famous Great Wave painting.".into(),
            palette: ThemePalette {
                bg: rgb(31, 31, 40),
                bg_alt: rgb(22, 22, 29),
                bg_highlight: rgb(42, 42, 55),
                fg: rgb(220, 215, 186),
                comment: rgb(114, 113, 105),
                keyword: rgb(149, 127, 184),
                func: rgb(126, 156, 216),
                string: rgb(152, 187, 108),
                number: rgb(210, 126, 153),
                r#type: rgb(122, 168, 159),
                macro_reg: rgb(228, 104, 118),
                operator: rgb(192, 163, 110),
                accent: rgb(126, 156, 216),
            },
            installed: false,
            is_builtin: false,
        },
        // 9. Kanagawa Dragon
        ThemeEntry {
            display_name: "Kanagawa Dragon".into(),
            colorscheme: "kanagawa-dragon".into(),
            repo: "rebelot/kanagawa.nvim".into(),
            lazy_dir: "kanagawa.nvim".into(),
            plugin_name_override: None,
            dotfyle_rank: Some(3),
            is_light: false,
            description: "Warmer, pitch-dark charcoal variant of Kanagawa for late-night coding.".into(),
            palette: ThemePalette {
                bg: rgb(24, 22, 22),
                bg_alt: rgb(18, 18, 15),
                bg_highlight: rgb(40, 39, 39),
                fg: rgb(197, 201, 197),
                comment: rgb(115, 124, 115),
                keyword: rgb(137, 146, 167),
                func: rgb(139, 164, 176),
                string: rgb(138, 154, 123),
                number: rgb(162, 146, 163),
                r#type: rgb(142, 164, 162),
                macro_reg: rgb(196, 116, 110),
                operator: rgb(196, 178, 138),
                accent: rgb(196, 116, 110),
            },
            installed: false,
            is_builtin: false,
        },
        // 10. Rosé Pine Main (Dotfyle #4)
        ThemeEntry {
            display_name: "Rosé Pine".into(),
            colorscheme: "rose-pine".into(),
            repo: "rose-pine/neovim".into(),
            lazy_dir: "rose-pine".into(),
            plugin_name_override: Some("rose-pine".into()),
            dotfyle_rank: Some(4),
            is_light: false,
            description: "All natural pine, faux fur and a bit of soho vibes for the classy minimalist.".into(),
            palette: ThemePalette {
                bg: rgb(25, 23, 36),
                bg_alt: rgb(31, 29, 46),
                bg_highlight: rgb(38, 35, 58),
                fg: rgb(224, 222, 244),
                comment: rgb(110, 106, 134),
                keyword: rgb(49, 116, 143),
                func: rgb(235, 188, 186),
                string: rgb(246, 193, 119),
                number: rgb(196, 167, 231),
                r#type: rgb(156, 207, 216),
                macro_reg: rgb(235, 111, 146),
                operator: rgb(144, 140, 170),
                accent: rgb(235, 188, 186),
            },
            installed: false,
            is_builtin: false,
        },
        // 11. Rosé Pine Moon
        ThemeEntry {
            display_name: "Rosé Pine Moon".into(),
            colorscheme: "rose-pine-moon".into(),
            repo: "rose-pine/neovim".into(),
            lazy_dir: "rose-pine".into(),
            plugin_name_override: Some("rose-pine".into()),
            dotfyle_rank: Some(4),
            is_light: false,
            description: "Deep plum-tinted moonlight variant of Rosé Pine.".into(),
            palette: ThemePalette {
                bg: rgb(35, 33, 54),
                bg_alt: rgb(42, 39, 63),
                bg_highlight: rgb(57, 53, 82),
                fg: rgb(224, 222, 244),
                comment: rgb(110, 106, 134),
                keyword: rgb(62, 143, 176),
                func: rgb(234, 154, 151),
                string: rgb(246, 193, 119),
                number: rgb(196, 167, 231),
                r#type: rgb(156, 207, 216),
                macro_reg: rgb(235, 111, 146),
                operator: rgb(144, 140, 170),
                accent: rgb(196, 167, 231),
            },
            installed: false,
            is_builtin: false,
        },
        // 12. Gruvbox Dark (Dotfyle #5)
        ThemeEntry {
            display_name: "Gruvbox Dark".into(),
            colorscheme: "gruvbox".into(),
            repo: "ellisonleao/gruvbox.nvim".into(),
            lazy_dir: "gruvbox.nvim".into(),
            plugin_name_override: None,
            dotfyle_rank: Some(5),
            is_light: false,
            description: "Retro groove warm high-contrast colorscheme beloved by systems & C programmers.".into(),
            palette: ThemePalette {
                bg: rgb(40, 40, 40),
                bg_alt: rgb(29, 32, 33),
                bg_highlight: rgb(60, 56, 54),
                fg: rgb(235, 219, 178),
                comment: rgb(146, 131, 116),
                keyword: rgb(251, 73, 52),
                func: rgb(184, 187, 38),
                string: rgb(184, 187, 38),
                number: rgb(211, 134, 155),
                r#type: rgb(250, 189, 47),
                macro_reg: rgb(254, 128, 25),
                operator: rgb(142, 192, 124),
                accent: rgb(254, 128, 25),
            },
            installed: false,
            is_builtin: false,
        },
        // 13. Nightfox (Dotfyle #6)
        ThemeEntry {
            display_name: "Nightfox".into(),
            colorscheme: "nightfox".into(),
            repo: "EdenEast/nightfox.nvim".into(),
            lazy_dir: "nightfox.nvim".into(),
            plugin_name_override: None,
            dotfyle_rank: Some(6),
            is_light: false,
            description: "Highly customizable dark ocean-blue theme from the Nightfox family.".into(),
            palette: ThemePalette {
                bg: rgb(25, 35, 48),
                bg_alt: rgb(19, 26, 36),
                bg_highlight: rgb(41, 57, 79),
                fg: rgb(205, 206, 207),
                comment: rgb(113, 131, 155),
                keyword: rgb(157, 121, 214),
                func: rgb(113, 156, 214),
                string: rgb(129, 178, 154),
                number: rgb(244, 162, 97),
                r#type: rgb(219, 192, 116),
                macro_reg: rgb(201, 79, 109),
                operator: rgb(99, 205, 207),
                accent: rgb(113, 156, 214),
            },
            installed: false,
            is_builtin: false,
        },
        // 14. Carbonfox
        ThemeEntry {
            display_name: "Carbonfox".into(),
            colorscheme: "carbonfox".into(),
            repo: "EdenEast/nightfox.nvim".into(),
            lazy_dir: "nightfox.nvim".into(),
            plugin_name_override: None,
            dotfyle_rank: Some(6),
            is_light: false,
            description: "Ultra-crisp IBM Carbon inspired deep dark theme from Nightfox.".into(),
            palette: ThemePalette {
                bg: rgb(22, 22, 22),
                bg_alt: rgb(18, 18, 18),
                bg_highlight: rgb(40, 40, 40),
                fg: rgb(242, 244, 248),
                comment: rgb(123, 124, 126),
                keyword: rgb(190, 149, 255),
                func: rgb(120, 169, 255),
                string: rgb(37, 190, 106),
                number: rgb(61, 219, 217),
                r#type: rgb(51, 177, 255),
                macro_reg: rgb(238, 83, 150),
                operator: rgb(8, 189, 186),
                accent: rgb(120, 169, 255),
            },
            installed: false,
            is_builtin: false,
        },
        // 15. Nordfox
        ThemeEntry {
            display_name: "Nordfox".into(),
            colorscheme: "nordfox".into(),
            repo: "EdenEast/nightfox.nvim".into(),
            lazy_dir: "nightfox.nvim".into(),
            plugin_name_override: None,
            dotfyle_rank: Some(6),
            is_light: false,
            description: "Arctic bluish-gray Nord palette with full Treesitter & LSP support.".into(),
            palette: ThemePalette {
                bg: rgb(46, 52, 64),
                bg_alt: rgb(35, 40, 49),
                bg_highlight: rgb(59, 66, 82),
                fg: rgb(205, 206, 207),
                comment: rgb(96, 114, 138),
                keyword: rgb(180, 142, 173),
                func: rgb(129, 161, 193),
                string: rgb(163, 190, 140),
                number: rgb(208, 135, 112),
                r#type: rgb(235, 203, 139),
                macro_reg: rgb(191, 97, 106),
                operator: rgb(136, 192, 208),
                accent: rgb(136, 192, 208),
            },
            installed: false,
            is_builtin: false,
        },
        // 16. OneDark Pro (Dotfyle #7)
        ThemeEntry {
            display_name: "OneDark Pro".into(),
            colorscheme: "onedark".into(),
            repo: "olimorris/onedarkpro.nvim".into(),
            lazy_dir: "onedarkpro.nvim".into(),
            plugin_name_override: None,
            dotfyle_rank: Some(7),
            is_light: false,
            description: "Atom's iconic One Dark theme supercharged for Neovim Treesitter & LSP.".into(),
            palette: ThemePalette {
                bg: rgb(40, 44, 52),
                bg_alt: rgb(33, 37, 43),
                bg_highlight: rgb(44, 49, 60),
                fg: rgb(171, 178, 191),
                comment: rgb(92, 99, 112),
                keyword: rgb(198, 120, 221),
                func: rgb(97, 175, 239),
                string: rgb(152, 195, 121),
                number: rgb(209, 154, 102),
                r#type: rgb(229, 192, 123),
                macro_reg: rgb(224, 108, 117),
                operator: rgb(86, 182, 194),
                accent: rgb(97, 175, 239),
            },
            installed: false,
            is_builtin: false,
        },
        // 17. Everforest Dark (Dotfyle #8)
        ThemeEntry {
            display_name: "Everforest Dark".into(),
            colorscheme: "everforest".into(),
            repo: "neanias/everforest-nvim".into(),
            lazy_dir: "everforest-nvim".into(),
            plugin_name_override: None,
            dotfyle_rank: Some(8),
            is_light: false,
            description: "Comfortable, eye-friendly green forest color palette designed for long sessions.".into(),
            palette: ThemePalette {
                bg: rgb(45, 53, 59),
                bg_alt: rgb(35, 42, 46),
                bg_highlight: rgb(52, 63, 68),
                fg: rgb(211, 198, 170),
                comment: rgb(133, 146, 137),
                keyword: rgb(230, 126, 128),
                func: rgb(167, 192, 128),
                string: rgb(167, 192, 128),
                number: rgb(214, 153, 182),
                r#type: rgb(219, 188, 127),
                macro_reg: rgb(230, 152, 117),
                operator: rgb(131, 192, 146),
                accent: rgb(167, 192, 128),
            },
            installed: false,
            is_builtin: false,
        },
        // 18. Cyberdream (Dotfyle #9)
        ThemeEntry {
            display_name: "Cyberdream".into(),
            colorscheme: "cyberdream".into(),
            repo: "scottmckendry/cyberdream.nvim".into(),
            lazy_dir: "cyberdream.nvim".into(),
            plugin_name_override: None,
            dotfyle_rank: Some(9),
            is_light: false,
            description: "High-contrast futuristic neon dark theme built for speed and clarity.".into(),
            palette: ThemePalette {
                bg: rgb(22, 24, 26),
                bg_alt: rgb(18, 20, 22),
                bg_highlight: rgb(44, 48, 54),
                fg: rgb(255, 255, 255),
                comment: rgb(123, 132, 150),
                keyword: rgb(255, 110, 94),
                func: rgb(94, 161, 255),
                string: rgb(94, 255, 108),
                number: rgb(255, 189, 94),
                r#type: rgb(94, 241, 255),
                macro_reg: rgb(255, 94, 241),
                operator: rgb(189, 94, 255),
                accent: rgb(94, 241, 255),
            },
            installed: false,
            is_builtin: false,
        },
        // 19. GitHub Dark Default (Dotfyle #10)
        ThemeEntry {
            display_name: "GitHub Dark Default".into(),
            colorscheme: "github_dark_default".into(),
            repo: "projekt0n/github-nvim-theme".into(),
            lazy_dir: "github-nvim-theme".into(),
            plugin_name_override: None,
            dotfyle_rank: Some(10),
            is_light: false,
            description: "Official GitHub Dark Default look ported faithfully to Neovim.".into(),
            palette: ThemePalette {
                bg: rgb(13, 17, 23),
                bg_alt: rgb(1, 4, 9),
                bg_highlight: rgb(22, 27, 34),
                fg: rgb(230, 237, 243),
                comment: rgb(139, 148, 158),
                keyword: rgb(255, 123, 114),
                func: rgb(210, 168, 255),
                string: rgb(165, 214, 255),
                number: rgb(121, 192, 255),
                r#type: rgb(255, 166, 87),
                macro_reg: rgb(121, 192, 255),
                operator: rgb(255, 123, 114),
                accent: rgb(88, 166, 255),
            },
            installed: false,
            is_builtin: false,
        },
        // 20. Dracula (Dotfyle #11)
        ThemeEntry {
            display_name: "Dracula".into(),
            colorscheme: "dracula".into(),
            repo: "Mofiqul/dracula.nvim".into(),
            lazy_dir: "dracula.nvim".into(),
            plugin_name_override: None,
            dotfyle_rank: Some(11),
            is_light: false,
            description: "Classic vampire purple & neon-pink dark theme for Neovim.".into(),
            palette: ThemePalette {
                bg: rgb(40, 42, 54),
                bg_alt: rgb(33, 34, 44),
                bg_highlight: rgb(68, 71, 90),
                fg: rgb(248, 248, 242),
                comment: rgb(98, 114, 164),
                keyword: rgb(255, 121, 198),
                func: rgb(80, 250, 123),
                string: rgb(241, 250, 140),
                number: rgb(189, 147, 249),
                r#type: rgb(139, 233, 253),
                macro_reg: rgb(255, 184, 108),
                operator: rgb(255, 121, 198),
                accent: rgb(189, 147, 249),
            },
            installed: false,
            is_builtin: false,
        },
        // 21. Habamax (Built-in fallback)
        ThemeEntry {
            display_name: "Habamax (Neovim Built-in)".into(),
            colorscheme: "habamax".into(),
            repo: "builtin".into(),
            lazy_dir: String::new(),
            plugin_name_override: None,
            dotfyle_rank: None,
            is_light: false,
            description: "Built-in retro dark colorscheme included out-of-the-box with Neovim.".into(),
            palette: ThemePalette {
                bg: rgb(28, 28, 28),
                bg_alt: rgb(18, 18, 18),
                bg_highlight: rgb(48, 48, 48),
                fg: rgb(188, 188, 188),
                comment: rgb(118, 118, 118),
                keyword: rgb(175, 135, 175),
                func: rgb(135, 175, 215),
                string: rgb(135, 175, 135),
                number: rgb(215, 135, 95),
                r#type: rgb(135, 175, 175),
                macro_reg: rgb(215, 95, 95),
                operator: rgb(188, 188, 188),
                accent: rgb(135, 175, 215),
            },
            installed: true,
            is_builtin: true,
        },
    ]
}
