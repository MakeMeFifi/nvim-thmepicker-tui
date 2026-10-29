use ratatui::style::Color;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

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
    pub stars: Option<u32>,
    pub description: String,
    pub palette: ThemePalette,
    pub installed: bool,
    pub is_builtin: bool,
}

impl ThemeEntry {
    pub fn stars_formatted(&self) -> String {
        match self.stars {
            Some(s) if s >= 1000 => format!("★ {:.1}k", (s as f64) / 1000.0),
            Some(s) => format!("★ {}", s),
            None => String::new(),
        }
    }
}

/// Fetch a page of colorschemes live from `https://dotfyle.com/neovim/colorscheme/top?page=<page>`
/// and convert each entry into a `ThemeEntry` with an accurate palette and `:colorscheme` name.
pub fn fetch_dotfyle_page(page: usize, start_rank: usize, lazy_root: &Path) -> Vec<ThemeEntry> {
    let url = format!("https://dotfyle.com/neovim/colorscheme/top?page={}", page);
    fetch_dotfyle_url(&url, Some(start_rank), lazy_root)
}

/// Search colorschemes live on `https://dotfyle.com/neovim/colorscheme/top?q=<query>`
/// and return all matching `ThemeEntry` items from Dotfyle.
pub fn search_dotfyle_remote(query: &str, lazy_root: &Path) -> Vec<ThemeEntry> {
    let encoded = url_encode(query.trim());
    if encoded.is_empty() {
        return Vec::new();
    }
    let url = format!("https://dotfyle.com/neovim/colorscheme/top?q={}", encoded);
    fetch_dotfyle_url(&url, None, lazy_root)
}

fn url_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for b in input.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else if b == b' ' {
            out.push('+');
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

fn fetch_dotfyle_url(url: &str, start_rank: Option<usize>, lazy_root: &Path) -> Vec<ThemeEntry> {
    let output = Command::new("curl")
        .args([
            "-fsSL",
            "--connect-timeout",
            "5",
            "--max-time",
            "8",
            "-A",
            "Mozilla/5.0 (X11; Linux x86_64) nvim-theme-tui/0.3.0",
            url,
        ])
        .output();

    let Ok(out) = output else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }

    let html = String::from_utf8_lossy(&out.stdout);
    parse_dotfyle_html(&html, start_rank, lazy_root)
}

fn parse_dotfyle_html(
    html: &str,
    start_rank: Option<usize>,
    lazy_root: &Path,
) -> Vec<ThemeEntry> {
    let mut results = Vec::new();
    let mut seen_repos = HashSet::new();
    let marker = "\\\"owner\\\":\\\"";
    let mut search_idx = 0;
    let mut current_rank = start_rank;

    while let Some(pos) = html[search_idx..].find(marker) {
        let abs_pos = search_idx + pos + marker.len();
        search_idx = abs_pos;

        let chunk_end = (abs_pos + 1800).min(html.len());
        let chunk = &html[abs_pos..chunk_end];

        let Some(owner) = extract_until(chunk, "\\\"") else {
            continue;
        };
        let Some(name) = extract_field(chunk, "\\\"name\\\":\\\"") else {
            continue;
        };
        let category = extract_field(chunk, "\\\"category\\\":\\\"").unwrap_or_default();
        if category != "colorscheme" {
            continue;
        }

        let repo = format!("{}/{}", owner, name);
        if !seen_repos.insert(repo.clone()) {
            continue;
        }

        let short_desc = extract_field(chunk, "\\\"shortDescription\\\":\\\"")
            .unwrap_or_else(|| format!("Popular Neovim colorscheme by {}", owner));
        let clean_desc = unescape_json_str(&short_desc);
        let stars = extract_number_field(chunk, "\\\"stars\\\":");

        let (display_name, colorscheme, lazy_dir, plugin_override, _is_light, palette) =
            resolve_theme_preset(&owner, &name);

        let installed = lazy_root.join(&lazy_dir).is_dir();

        results.push(ThemeEntry {
            display_name,
            colorscheme,
            repo,
            lazy_dir,
            plugin_name_override: plugin_override,
            dotfyle_rank: current_rank,
            stars,
            description: if clean_desc.is_empty() {
                format!("Neovim colorscheme from github.com/{}/{}", owner, name)
            } else {
                clean_desc
            },
            palette,
            installed,
            is_builtin: false,
        });
        if let Some(r) = current_rank.as_mut() {
            *r += 1;
        }
    }

    results
}

fn extract_until<'a>(s: &'a str, delim: &str) -> Option<&'a str> {
    let idx = s.find(delim)?;
    Some(&s[..idx])
}

fn extract_field(chunk: &str, key: &str) -> Option<String> {
    let start = chunk.find(key)? + key.len();
    let rest = &chunk[start..];
    let end = rest.find("\\\"")?;
    Some(rest[..end].to_string())
}

fn extract_number_field(chunk: &str, key: &str) -> Option<u32> {
    let start = chunk.find(key)? + key.len();
    let rest = &chunk[start..];
    let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn unescape_json_str(s: &str) -> String {
    s.replace("\\n", " ")
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
        .trim()
        .to_string()
}

/// Detect the primary colorscheme name inside an installed plugin's `colors/` directory
pub fn detect_colorscheme_in_plugin_dir(plugin_dir: &Path, fallback: &str) -> String {
    let colors_dir = plugin_dir.join("colors");
    if !colors_dir.is_dir() {
        return fallback.to_string();
    }
    let mut found = Vec::new();
    if let Ok(entries) = fs::read_dir(&colors_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext == "lua" || ext == "vim" {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    found.push(stem.to_string());
                }
            }
        }
    }
    if found.is_empty() {
        return fallback.to_string();
    }
    found.sort();
    if found.iter().any(|s| s == fallback) {
        return fallback.to_string();
    }
    found[0].clone()
}

/// Resolves known Dotfyle repositories to their exact display title, `:colorscheme` name,
/// Lazy directory name, optional plugin name override, light/dark flag, and 24-bit RGB palette.
fn resolve_theme_preset(
    owner: &str,
    repo_name: &str,
) -> (String, String, String, Option<String>, bool, ThemePalette) {
    let full = format!("{}/{}", owner, repo_name);
    match full.as_str() {
        "catppuccin/nvim" => (
            "Catppuccin Mocha".into(),
            "catppuccin-mocha".into(),
            "catppuccin".into(),
            Some("catppuccin".into()),
            false,
            ThemePalette {
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
        ),
        "folke/tokyonight.nvim" => (
            "TokyoNight Night".into(),
            "tokyonight-night".into(),
            "tokyonight.nvim".into(),
            None,
            false,
            default_dark_palette(),
        ),
        "rebelot/kanagawa.nvim" => (
            "Kanagawa Wave".into(),
            "kanagawa-wave".into(),
            "kanagawa.nvim".into(),
            None,
            false,
            ThemePalette {
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
        ),
        "rose-pine/neovim" => (
            "Rosé Pine".into(),
            "rose-pine".into(),
            "rose-pine".into(),
            Some("rose-pine".into()),
            false,
            ThemePalette {
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
        ),
        "EdenEast/nightfox.nvim" => (
            "Nightfox".into(),
            "nightfox".into(),
            "nightfox.nvim".into(),
            None,
            false,
            ThemePalette {
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
        ),
        "navarasu/onedark.nvim" | "olimorris/onedarkpro.nvim" => (
            if owner == "navarasu" {
                "OneDark".into()
            } else {
                "OneDark Pro".into()
            },
            "onedark".into(),
            repo_name.into(),
            None,
            false,
            ThemePalette {
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
        ),
        "ellisonleao/gruvbox.nvim" | "sainnhe/gruvbox-material" | "luisiacc/gruvbox-baby" => {
            let cs = if repo_name == "gruvbox-material" {
                "gruvbox-material"
            } else if repo_name == "gruvbox-baby" {
                "gruvbox-baby"
            } else {
                "gruvbox"
            };
            (
                prettify_name(cs),
                cs.into(),
                repo_name.into(),
                None,
                false,
                ThemePalette {
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
            )
        }
        "projekt0n/github-nvim-theme" => (
            "GitHub Dark Default".into(),
            "github_dark_default".into(),
            "github-nvim-theme".into(),
            None,
            false,
            ThemePalette {
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
        ),
        "sainnhe/everforest" | "neanias/everforest-nvim" => (
            if owner == "sainnhe" {
                "Everforest".into()
            } else {
                "Everforest Lua".into()
            },
            "everforest".into(),
            repo_name.into(),
            None,
            false,
            ThemePalette {
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
        ),
        "scottmckendry/cyberdream.nvim" => (
            "Cyberdream".into(),
            "cyberdream".into(),
            "cyberdream.nvim".into(),
            None,
            false,
            ThemePalette {
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
        ),
        "Mofiqul/vscode.nvim" => (
            "VSCode Dark+".into(),
            "vscode".into(),
            "vscode.nvim".into(),
            None,
            false,
            ThemePalette {
                bg: rgb(30, 30, 30),
                bg_alt: rgb(24, 24, 24),
                bg_highlight: rgb(45, 45, 48),
                fg: rgb(212, 212, 212),
                comment: rgb(106, 153, 85),
                keyword: rgb(86, 156, 214),
                func: rgb(220, 220, 170),
                string: rgb(206, 145, 120),
                number: rgb(181, 206, 168),
                r#type: rgb(78, 201, 176),
                macro_reg: rgb(197, 134, 192),
                operator: rgb(212, 212, 212),
                accent: rgb(86, 156, 214),
            },
        ),
        "Mofiqul/dracula.nvim" | "dracula/vim" => (
            "Dracula".into(),
            "dracula".into(),
            repo_name.into(),
            None,
            false,
            ThemePalette {
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
        ),
        "shaunsingh/nord.nvim" | "AlexvZyl/nordic.nvim" | "rmehri01/onenord.nvim" => {
            let cs = repo_name.trim_end_matches(".nvim");
            (
                prettify_name(cs),
                cs.into(),
                repo_name.into(),
                None,
                false,
                ThemePalette {
                    bg: rgb(46, 52, 64),
                    bg_alt: rgb(36, 41, 51),
                    bg_highlight: rgb(59, 66, 82),
                    fg: rgb(216, 222, 233),
                    comment: rgb(97, 110, 136),
                    keyword: rgb(129, 161, 193),
                    func: rgb(136, 192, 208),
                    string: rgb(163, 190, 140),
                    number: rgb(180, 142, 173),
                    r#type: rgb(143, 188, 187),
                    macro_reg: rgb(191, 97, 106),
                    operator: rgb(129, 161, 193),
                    accent: rgb(136, 192, 208),
                },
            )
        }
        "craftzdog/solarized-osaka.nvim" => (
            "Solarized Osaka".into(),
            "solarized-osaka".into(),
            "solarized-osaka.nvim".into(),
            None,
            false,
            ThemePalette {
                bg: rgb(0, 20, 26),
                bg_alt: rgb(0, 14, 19),
                bg_highlight: rgb(7, 54, 66),
                fg: rgb(147, 161, 161),
                comment: rgb(88, 110, 117),
                keyword: rgb(133, 153, 0),
                func: rgb(38, 139, 210),
                string: rgb(42, 161, 152),
                number: rgb(211, 54, 130),
                r#type: rgb(181, 137, 0),
                macro_reg: rgb(203, 75, 22),
                operator: rgb(133, 153, 0),
                accent: rgb(38, 139, 210),
            },
        ),
        "nyoom-engineering/oxocarbon.nvim" => (
            "Oxocarbon".into(),
            "oxocarbon".into(),
            "oxocarbon.nvim".into(),
            None,
            false,
            ThemePalette {
                bg: rgb(22, 22, 22),
                bg_alt: rgb(15, 15, 15),
                bg_highlight: rgb(38, 38, 38),
                fg: rgb(242, 244, 248),
                comment: rgb(105, 112, 119),
                keyword: rgb(255, 126, 182),
                func: rgb(120, 169, 255),
                string: rgb(66, 190, 101),
                number: rgb(130, 207, 255),
                r#type: rgb(51, 177, 255),
                macro_reg: rgb(190, 149, 255),
                operator: rgb(8, 189, 186),
                accent: rgb(255, 126, 182),
            },
        ),
        "bluz71/vim-moonfly-colors" => (
            "Moonfly".into(),
            "moonfly".into(),
            "vim-moonfly-colors".into(),
            None,
            false,
            synthesized_palette("moonfly", 210),
        ),
        "bluz71/vim-nightfly-colors" => (
            "Nightfly".into(),
            "nightfly".into(),
            "vim-nightfly-colors".into(),
            None,
            false,
            synthesized_palette("nightfly", 225),
        ),
        _ => {
            let clean_scheme = derive_colorscheme_name(owner, repo_name);
            let title = prettify_name(&clean_scheme);
            let seed = full.bytes().fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
            (
                title,
                clean_scheme,
                repo_name.to_string(),
                None,
                false,
                synthesized_palette(&full, (seed % 360) as u16),
            )
        }
    }
}

fn derive_colorscheme_name(owner: &str, repo_name: &str) -> String {
    if repo_name == "nvim" || repo_name == "neovim" || repo_name == "vim" {
        return owner.to_lowercase();
    }
    repo_name
        .trim_start_matches("vim-")
        .trim_start_matches("nvim-")
        .trim_end_matches(".nvim")
        .trim_end_matches("-nvim")
        .trim_end_matches(".vim")
        .trim_end_matches("-vim")
        .trim_end_matches("-colors")
        .to_string()
}

fn prettify_name(slug: &str) -> String {
    slug.split(['-', '_'])
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn synthesized_palette(_key: &str, hue: u16) -> ThemePalette {
    let h = (hue % 360) as f32;
    let kw = hsl_to_rgb((h + 280.0) % 360.0, 0.75, 0.72);
    let fn_c = hsl_to_rgb((h + 200.0) % 360.0, 0.80, 0.72);
    let st_c = hsl_to_rgb((h + 110.0) % 360.0, 0.65, 0.70);
    let num_c = hsl_to_rgb((h + 30.0) % 360.0, 0.85, 0.72);
    let ty_c = hsl_to_rgb((h + 55.0) % 360.0, 0.80, 0.74);
    let mac_c = hsl_to_rgb((h + 345.0) % 360.0, 0.80, 0.70);
    let op_c = hsl_to_rgb((h + 180.0) % 360.0, 0.75, 0.74);
    let bg = hsl_to_rgb(h, 0.22, 0.13);
    let bg_alt = hsl_to_rgb(h, 0.24, 0.10);
    let bg_hi = hsl_to_rgb(h, 0.24, 0.20);
    let fg = hsl_to_rgb(h, 0.25, 0.88);
    let comment = hsl_to_rgb(h, 0.16, 0.48);

    ThemePalette {
        bg,
        bg_alt,
        bg_highlight: bg_hi,
        fg,
        comment,
        keyword: kw,
        func: fn_c,
        string: st_c,
        number: num_c,
        r#type: ty_c,
        macro_reg: mac_c,
        operator: op_c,
        accent: fn_c,
    }
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r1, g1, b1) = match h as u16 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    Color::Rgb(
        ((r1 + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((g1 + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((b1 + m) * 255.0).clamp(0.0, 255.0) as u8,
    )
}

/// Load initial themes (fetches page 1 live from Dotfyle, with offline fallback,
/// plus any locally installed LazyVim themes).
pub fn load_initial_themes(lazy_root: &Path) -> (Vec<ThemeEntry>, usize) {
    let mut entries = fetch_dotfyle_page(1, 1, lazy_root);
    let fetched_page = if entries.is_empty() {
        entries = offline_fallback_themes();
        for e in &mut entries {
            e.installed = e.is_builtin || lazy_root.join(&e.lazy_dir).is_dir();
        }
        0
    } else {
        1
    };

    append_local_installed_themes(&mut entries, lazy_root);
    (entries, fetched_page)
}

pub fn append_local_installed_themes(entries: &mut Vec<ThemeEntry>, lazy_root: &Path) {
    let mut known_repos: HashSet<String> = entries.iter().map(|e| e.repo.to_lowercase()).collect();
    let mut known_schemes: HashSet<String> = entries
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
            if known_repos.contains(&repo_guess.to_lowercase()) {
                continue;
            }

            if let Ok(color_files) = fs::read_dir(&colors_dir) {
                for cf in color_files.flatten() {
                    let path = cf.path();
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if ext != "lua" && ext != "vim" {
                        continue;
                    }
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if known_schemes.insert(stem.to_lowercase()) {
                            known_repos.insert(repo_guess.to_lowercase());
                            entries.push(ThemeEntry {
                                display_name: prettify_name(stem),
                                colorscheme: stem.to_string(),
                                repo: repo_guess.clone(),
                                lazy_dir: plugin_dir_name.clone(),
                                plugin_name_override: None,
                                dotfyle_rank: None,
                                stars: None,
                                description: format!(
                                    "Locally installed colorscheme in ~/.local/share/nvim/lazy/{}",
                                    plugin_dir_name
                                ),
                                palette: default_dark_palette(),
                                installed: true,
                                is_builtin: false,
                            });
                            break;
                        }
                    }
                }
            }
        }
    }
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

fn offline_fallback_themes() -> Vec<ThemeEntry> {
    let fallback_repos = [
        ("catppuccin", "nvim", "Soothing pastel theme for Neovim", 7652),
        ("folke", "tokyonight.nvim", "Clean, dark Neovim theme written in Lua", 8230),
        ("rebelot", "kanagawa.nvim", "Dark colorscheme inspired by Hokusai's Great Wave", 6430),
        ("rose-pine", "neovim", "All natural pine, faux fur and soho vibes", 3107),
        ("EdenEast", "nightfox.nvim", "Highly customizable theme for Vim and Neovim", 4093),
        ("navarasu", "onedark.nvim", "Atom's One Dark and Light theme for Neovim", 2012),
        ("sainnhe", "gruvbox-material", "Gruvbox with Material Palette", 2655),
        ("projekt0n", "github-nvim-theme", "GitHub's Neovim themes", 2475),
        ("sainnhe", "everforest", "Comfortable & pleasant green forest color scheme", 4262),
        ("scottmckendry", "cyberdream.nvim", "High-contrast, futuristic & vibrant colorscheme", 1422),
        ("Mofiqul", "vscode.nvim", "Dark+ and Light+ theme inspired by VS Code", 1002),
        ("olimorris", "onedarkpro.nvim", "Atom's iconic One Dark theme for Neovim", 1099),
        ("Mofiqul", "dracula.nvim", "Dracula colorscheme for Neovim written in Lua", 1150),
        ("shaunsingh", "nord.nvim", "Arctic, north-bluish clean Neovim theme", 1820),
        ("craftzdog", "solarized-osaka.nvim", "Clean dark Solarized theme for Neovim", 1340),
        ("nyoom-engineering", "oxocarbon.nvim", "Dark and light Neovim theme inspired by IBM Carbon", 1490),
    ];

    fallback_repos
        .iter()
        .enumerate()
        .map(|(i, (owner, name, desc, stars))| {
            let (display_name, colorscheme, lazy_dir, plugin_override, _is_light, palette) =
                resolve_theme_preset(owner, name);
            ThemeEntry {
                display_name,
                colorscheme,
                repo: format!("{}/{}", owner, name),
                lazy_dir,
                plugin_name_override: plugin_override,
                dotfyle_rank: Some(i + 1),
                stars: Some(*stars),
                description: (*desc).to_string(),
                palette,
                installed: false,
                is_builtin: false,
            }
        })
        .collect()
}
