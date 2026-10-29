use crate::app::{App, FilterTab, ListRow, PreviewLanguage};
use crate::themes::{ThemeEntry, ThemePalette};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    let root = frame.area();

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Clean Top Bar
            Constraint::Min(14),   // Main Split (Themes List | Live Code Preview)
            Constraint::Length(2), // Compact Status & Keymap Bar
        ])
        .split(root);

    draw_top_bar(frame, app, vertical[0]);

    let main_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(vertical[1]);

    draw_left_panel(frame, app, main_split[0]);
    draw_right_preview(frame, app, main_split[1]);
    draw_bottom_bar(frame, app, vertical[2]);

    if app.show_help {
        draw_help_modal(frame, root);
    }
}

fn draw_top_bar(frame: &mut Frame, app: &App, area: Rect) {
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(38),
            Constraint::Percentage(37),
            Constraint::Percentage(25),
        ])
        .split(area);

    // 1. App Title & Active Colorscheme Pill
    let title_line = Line::from(vec![
        Span::styled(
            " nvim-theme-tui ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("│ Active: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(" {} ", app.active_colorscheme),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let title_box = Paragraph::new(title_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::DarkGray)),
    );
    frame.render_widget(title_box, cols[0]);

    // 2. Filter Tabs
    let tab_titles = vec![
        FilterTab::All.title(),
        FilterTab::Installed.title(),
        FilterTab::DotfyleTop.title(),
    ];
    let selected_tab_idx = match app.tab {
        FilterTab::All => 0,
        FilterTab::Installed => 1,
        FilterTab::DotfyleTop => 2,
    };
    let tabs = Tabs::new(tab_titles)
        .select(selected_tab_idx)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    frame.render_widget(tabs, cols[1]);

    // 3. Search Box
    let search_border = if app.is_searching {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let search_str = if app.is_searching {
        format!(" / {}_ (Enter=Dotfyle)", app.search_query)
    } else if app.search_query.is_empty() {
        " [/] Search (Enter=Dotfyle)".to_string()
    } else {
        format!(" / {} (Esc=Clear)", app.search_query)
    };
    let search = Paragraph::new(search_str)
        .style(Style::default().fg(if app.is_searching || !app.search_query.is_empty() {
            Color::White
        } else {
            Color::DarkGray
        }))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(search_border)),
        );
    frame.render_widget(search, cols[2]);
}

fn draw_left_panel(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(5)])
        .split(area);

    let items: Vec<ListItem> = app
        .rows
        .iter()
        .map(|row| match row {
            ListRow::Theme(idx) => {
                let t = &app.themes[*idx];
                let is_active = t.colorscheme == app.active_colorscheme
                    || (app.active_colorscheme == "tokyonight" && t.colorscheme == "tokyonight-night");

                let rank_str = match t.dotfyle_rank {
                    Some(r) => format!("#{:02} ", r),
                    None if t.installed => "LOC ".to_string(),
                    None => "WEB ".to_string(),
                };

                let status_badge = if is_active {
                    Span::styled(
                        " ● ACTIVE",
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    )
                } else if t.installed {
                    Span::styled(" ✓ Installed", Style::default().fg(Color::Cyan))
                } else {
                    Span::styled(
                        format!(" {}", t.stars_formatted()),
                        Style::default().fg(Color::Yellow),
                    )
                };

                ListItem::new(Line::from(vec![
                    Span::styled(rank_str, Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{:<20}", truncate_str(&t.display_name, 20)),
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(if is_active {
                                Modifier::BOLD
                            } else {
                                Modifier::empty()
                            }),
                    ),
                    status_badge,
                ]))
            }
            ListRow::LoadMoreButton => ListItem::new(Line::from(vec![
                Span::styled(
                    "    [ + Load More (+10 from Dotfyle) ] ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            ])),
            ListRow::SearchDotfyleButton => ListItem::new(Line::from(vec![
                Span::styled(
                    format!(
                        "    [ 🌐 Search \"{}\" on Dotfyle.com ] ",
                        truncate_str(app.search_query.trim(), 16)
                    ),
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ])),
        })
        .collect();

    let visible_theme_count = app
        .rows
        .iter()
        .filter(|r| matches!(r, ListRow::Theme(_)))
        .count();

    let list_title = format!(" Themes (Showing {}) ", visible_theme_count);
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(list_title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 50, 75))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(" ▸ ");

    let mut state = ListState::default();
    if !app.rows.is_empty() {
        state.select(Some(app.selected_pos));
    }
    frame.render_stateful_widget(list, chunks[0], &mut state);

    // Compact 3-line Theme Info Card
    let info_lines = if matches!(app.current_row(), Some(ListRow::LoadMoreButton)) {
        vec![
            Line::from(Span::styled(
                " Press [Enter] or [m] to fetch 10 more themes from Dotfyle",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                " Source: https://dotfyle.com/neovim/colorscheme/top",
                Style::default().fg(Color::Gray),
            )),
        ]
    } else if matches!(app.current_row(), Some(ListRow::SearchDotfyleButton)) {
        vec![
            Line::from(Span::styled(
                format!(
                    " Press [Enter] to search & fetch \"{}\" live from Dotfyle.com",
                    app.search_query.trim()
                ),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                format!(
                    " Endpoint: https://dotfyle.com/neovim/colorscheme/top?q={}",
                    app.search_query.trim()
                ),
                Style::default().fg(Color::Gray),
            )),
        ]
    } else if let Some(t) = app.current_theme() {
        let status_label = if t.installed {
            Span::styled("Installed", Style::default().fg(Color::Green))
        } else {
            Span::styled("Auto-install on Enter", Style::default().fg(Color::Yellow))
        };
        vec![
            Line::from(vec![
                Span::styled(" Repo: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&t.repo, Style::default().fg(Color::Cyan)),
                Span::styled("  •  ", Style::default().fg(Color::DarkGray)),
                status_label,
            ]),
            Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled(
                    truncate_str(&t.description, 52),
                    Style::default().fg(Color::Gray),
                ),
            ]),
        ]
    } else {
        vec![Line::from(" No theme selected.")]
    };

    let info_card = Paragraph::new(info_lines)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Details ")
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    frame.render_widget(info_card, chunks[1]);
}

fn draw_right_preview(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(3)])
        .split(area);

    if let Some(theme) = app.current_theme() {
        draw_code_preview(frame, theme, app.preview_lang, chunks[0]);
        draw_palette_swatches(frame, theme, chunks[1]);
    } else {
        let empty = Paragraph::new("No preview available.").block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Live Preview "),
        );
        frame.render_widget(empty, area);
    }
}

fn draw_code_preview(
    frame: &mut Frame,
    theme: &ThemeEntry,
    lang: PreviewLanguage,
    area: Rect,
) {
    let p = &theme.palette;
    let code_lines = build_highlighted_snippet(p, lang, &theme.colorscheme);

    let title = format!(
        " Preview: {} (:colorscheme {}) │ Lang [l]: {} ",
        theme.display_name,
        theme.colorscheme,
        lang.label()
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .title(title)
        .style(Style::default().bg(p.bg).fg(p.fg))
        .border_style(Style::default().fg(p.accent));

    let paragraph = Paragraph::new(code_lines)
        .style(Style::default().bg(p.bg).fg(p.fg))
        .block(block);

    frame.render_widget(paragraph, area);
}

fn draw_palette_swatches(frame: &mut Frame, theme: &ThemeEntry, area: Rect) {
    let p = &theme.palette;
    let swatch_items = [
        ("BG", p.bg),
        ("FG", p.fg),
        ("Keyw", p.keyword),
        ("Func", p.func),
        ("Str", p.string),
        ("Type", p.r#type),
        ("Macro", p.macro_reg),
        ("Num", p.number),
        ("Comment", p.comment),
    ];

    let mut spans = Vec::new();
    for (name, color) in swatch_items {
        spans.push(Span::styled("██ ", Style::default().fg(color)));
        spans.push(Span::styled(
            format!("{}  ", name),
            Style::default().fg(Color::Gray),
        ));
    }

    let swatches = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
    frame.render_widget(swatches, area);
}

fn draw_bottom_bar(frame: &mut Frame, app: &App, area: Rect) {
    let status_style = if app.status_is_error {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(&app.status_message, status_style),
        ]),
        Line::from(vec![
            Span::styled(" ↑/↓", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" Select  ", Style::default().fg(Color::Gray)),
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" Apply  ", Style::default().fg(Color::Gray)),
            Span::styled("m", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" Load +10  ", Style::default().fg(Color::Gray)),
            Span::styled("Tab", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" Filter  ", Style::default().fg(Color::Gray)),
            Span::styled("l", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" Language  ", Style::default().fg(Color::Gray)),
            Span::styled("/", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(" Search  ", Style::default().fg(Color::Gray)),
            Span::styled("r", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            Span::styled(" Rollback  ", Style::default().fg(Color::Gray)),
            Span::styled("?", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled(" Help  ", Style::default().fg(Color::Gray)),
            Span::styled("q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" Quit", Style::default().fg(Color::Gray)),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines), area);
}

fn draw_help_modal(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(64, 62, area);
    frame.render_widget(Clear, popup_area);

    let lines = vec![
        Line::from(Span::styled(
            "Keyboard Shortcuts — nvim-theme-tui",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  ↑ / ↓ or j / k      Navigate through themes (with live preview)"),
        Line::from("  Enter or a          Safely install & activate selected theme"),
        Line::from("  m                   Load 10 more themes live from Dotfyle"),
        Line::from("  Tab or 1 / 2 / 3    Switch filter: All / Installed / Dotfyle Top"),
        Line::from("  l                   Cycle preview language (AVR C / Rust / Python / C++)"),
        Line::from("  / + query + Enter   Search & fetch themes live from Dotfyle.com"),
        Line::from("  r                   Rollback to previous backup (colorscheme.lua.bak)"),
        Line::from("  ?                   Toggle this help window"),
        Line::from("  q or Esc            Quit application"),
        Line::from(""),
        Line::from(Span::styled(
            "Zero-Breakage Safety Guarantees:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  • Writes only to ~/.config/nvim/lua/plugins/colorscheme.lua."),
        Line::from("  • Creates an automatic backup (colorscheme.lua.bak) before every change."),
        Line::from("  • Pre-clones remote plugins into ~/.local/share/nvim/lazy before activating."),
        Line::from("  • Wraps vim.cmd.colorscheme in pcall() with fallback to 'tokyonight'."),
    ];

    let modal = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Help [? / Esc to close] "),
        );
    frame.render_widget(modal, popup_area);
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    let count = s.chars().count();
    if count <= max_chars {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max_chars.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn build_highlighted_snippet<'a>(
    p: &ThemePalette,
    lang: PreviewLanguage,
    scheme_name: &str,
) -> Vec<Line<'a>> {
    let ln = |num: usize, active: bool| -> Span<'a> {
        if active {
            Span::styled(
                format!(" {:>2} ", num),
                Style::default()
                    .fg(p.accent)
                    .bg(p.bg_highlight)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::styled(format!(" {:>2} ", num), Style::default().fg(p.comment).bg(p.bg))
        }
    };

    let kw = Style::default().fg(p.keyword).add_modifier(Modifier::BOLD);
    let fn_s = Style::default().fg(p.func);
    let st_s = Style::default().fg(p.string);
    let num_s = Style::default().fg(p.number);
    let ty_s = Style::default().fg(p.r#type);
    let reg_s = Style::default().fg(p.macro_reg).add_modifier(Modifier::BOLD);
    let op_s = Style::default().fg(p.operator);
    let cm_s = Style::default().fg(p.comment).add_modifier(Modifier::ITALIC);
    let fg_s = Style::default().fg(p.fg);

    let mut lines: Vec<Line<'a>> = match lang {
        PreviewLanguage::AvrC => vec![
            Line::from(vec![
                ln(1, false),
                Span::styled("// Microchip Studio 7.0.2594 — ATmega644 ADC Driver", cm_s),
            ]),
            Line::from(vec![
                ln(2, false),
                Span::styled("#include ", reg_s),
                Span::styled("<avr/io.h>", st_s),
            ]),
            Line::from(vec![
                ln(3, false),
                Span::styled("#include ", reg_s),
                Span::styled("<util/delay.h>", st_s),
            ]),
            Line::from(vec![
                ln(4, false),
                Span::styled("#define ", reg_s),
                Span::styled("F_CPU ", reg_s),
                Span::styled("20000000UL", num_s),
            ]),
            Line::from(vec![ln(5, false)]),
            Line::from(vec![
                ln(6, false),
                Span::styled("void ", kw),
                Span::styled("adc_init", fn_s),
                Span::styled("(", fg_s),
                Span::styled("uint8_t ", ty_s),
                Span::styled("channel) {", fg_s),
            ]),
            Line::from(vec![
                ln(7, true),
                Span::styled(
                    "    DDRA ",
                    Style::default()
                        .fg(p.macro_reg)
                        .bg(p.bg_highlight)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("= ", Style::default().fg(p.operator).bg(p.bg_highlight)),
                Span::styled("0xFF", Style::default().fg(p.number).bg(p.bg_highlight)),
                Span::styled("; ", Style::default().fg(p.fg).bg(p.bg_highlight)),
                Span::styled(
                    "// Configure Port A as R-2R output",
                    Style::default().fg(p.comment).bg(p.bg_highlight),
                ),
            ]),
            Line::from(vec![
                ln(8, false),
                Span::styled("    ADMUX ", reg_s),
                Span::styled("= (", fg_s),
                Span::styled("1 ", num_s),
                Span::styled("<< ", op_s),
                Span::styled("REFS0", reg_s),
                Span::styled(") | (channel & ", fg_s),
                Span::styled("0x07", num_s),
                Span::styled(");", fg_s),
            ]),
            Line::from(vec![
                ln(9, false),
                Span::styled("    ADCSRA ", reg_s),
                Span::styled("|= (", op_s),
                Span::styled("1 ", num_s),
                Span::styled("<< ", op_s),
                Span::styled("ADEN", reg_s),
                Span::styled(") | (", fg_s),
                Span::styled("1 ", num_s),
                Span::styled("<< ", op_s),
                Span::styled("ADSC", reg_s),
                Span::styled(");", fg_s),
            ]),
            Line::from(vec![
                ln(10, false),
                Span::styled("    ", fg_s),
                Span::styled("_delay_ms", fn_s),
                Span::styled("(", fg_s),
                Span::styled("10", num_s),
                Span::styled(");", fg_s),
            ]),
            Line::from(vec![ln(11, false), Span::styled("}", fg_s)]),
            Line::from(vec![
                ln(12, false),
                Span::styled("int ", kw),
                Span::styled("main", fn_s),
                Span::styled("(", fg_s),
                Span::styled("void", kw),
                Span::styled(") { ", fg_s),
                Span::styled("adc_init", fn_s),
                Span::styled("(", fg_s),
                Span::styled("0", num_s),
                Span::styled("); ", fg_s),
                Span::styled("return ", kw),
                Span::styled("0", num_s),
                Span::styled("; }", fg_s),
            ]),
        ],
        PreviewLanguage::Rust => vec![
            Line::from(vec![
                ln(1, false),
                Span::styled("// Rust Systems Programming — Deterministic Telemetry", cm_s),
            ]),
            Line::from(vec![
                ln(2, false),
                Span::styled("use ", kw),
                Span::styled("std::time::", fg_s),
                Span::styled("Duration", ty_s),
                Span::styled(";", fg_s),
            ]),
            Line::from(vec![ln(3, false)]),
            Line::from(vec![
                ln(4, false),
                Span::styled("#[derive(Debug, Clone)]", reg_s),
            ]),
            Line::from(vec![
                ln(5, false),
                Span::styled("pub struct ", kw),
                Span::styled("SensorFrame ", ty_s),
                Span::styled("{", fg_s),
            ]),
            Line::from(vec![
                ln(6, false),
                Span::styled("    pub ", kw),
                Span::styled("id: ", fg_s),
                Span::styled("u16", ty_s),
                Span::styled(",", fg_s),
            ]),
            Line::from(vec![
                ln(7, true),
                Span::styled("    pub ", Style::default().fg(p.keyword).bg(p.bg_highlight)),
                Span::styled("voltage_mv: ", Style::default().fg(p.fg).bg(p.bg_highlight)),
                Span::styled("f32", Style::default().fg(p.r#type).bg(p.bg_highlight)),
                Span::styled(",", Style::default().fg(p.fg).bg(p.bg_highlight)),
            ]),
            Line::from(vec![ln(8, false), Span::styled("}", fg_s)]),
            Line::from(vec![
                ln(9, false),
                Span::styled("pub fn ", kw),
                Span::styled("sample_adc", fn_s),
                Span::styled("(raw: ", fg_s),
                Span::styled("u16", ty_s),
                Span::styled(") -> ", op_s),
                Span::styled("SensorFrame ", ty_s),
                Span::styled("{", fg_s),
            ]),
            Line::from(vec![
                ln(10, false),
                Span::styled("    let ", kw),
                Span::styled("voltage_mv ", fg_s),
                Span::styled("= ", op_s),
                Span::styled("(raw ", fg_s),
                Span::styled("as ", kw),
                Span::styled("f32", ty_s),
                Span::styled(") * ", fg_s),
                Span::styled("4.8828", num_s),
                Span::styled(";", fg_s),
            ]),
            Line::from(vec![
                ln(11, false),
                Span::styled("    SensorFrame ", ty_s),
                Span::styled("{ id: ", fg_s),
                Span::styled("1", num_s),
                Span::styled(", voltage_mv }", fg_s),
            ]),
            Line::from(vec![ln(12, false), Span::styled("}", fg_s)]),
        ],
        PreviewLanguage::Python => vec![
            Line::from(vec![
                ln(1, false),
                Span::styled("# Python 3.14 + uv + basedpyright Type-Checked Pipeline", cm_s),
            ]),
            Line::from(vec![
                ln(2, false),
                Span::styled("from ", kw),
                Span::styled("dataclasses ", fg_s),
                Span::styled("import ", kw),
                Span::styled("dataclass", ty_s),
            ]),
            Line::from(vec![ln(3, false)]),
            Line::from(vec![ln(4, false), Span::styled("@dataclass", reg_s)]),
            Line::from(vec![
                ln(5, false),
                Span::styled("class ", kw),
                Span::styled("DeviceConfig", ty_s),
                Span::styled(":", fg_s),
            ]),
            Line::from(vec![
                ln(6, false),
                Span::styled("    mcu: ", fg_s),
                Span::styled("str ", ty_s),
                Span::styled("= ", op_s),
                Span::styled("\"ATmega644\"", st_s),
            ]),
            Line::from(vec![
                ln(7, true),
                Span::styled("    f_cpu: ", Style::default().fg(p.fg).bg(p.bg_highlight)),
                Span::styled("int ", Style::default().fg(p.r#type).bg(p.bg_highlight)),
                Span::styled("= ", Style::default().fg(p.operator).bg(p.bg_highlight)),
                Span::styled("20_000_000", Style::default().fg(p.number).bg(p.bg_highlight)),
            ]),
            Line::from(vec![ln(8, false)]),
            Line::from(vec![
                ln(9, false),
                Span::styled("def ", kw),
                Span::styled("baud_prescaler", fn_s),
                Span::styled("(cfg: ", fg_s),
                Span::styled("DeviceConfig", ty_s),
                Span::styled(", baud: ", fg_s),
                Span::styled("int", ty_s),
                Span::styled(") -> ", op_s),
                Span::styled("int", ty_s),
                Span::styled(":", fg_s),
            ]),
            Line::from(vec![
                ln(10, false),
                Span::styled("    return ", kw),
                Span::styled("(cfg.f_cpu // (", fg_s),
                Span::styled("16 ", num_s),
                Span::styled("* baud)) - ", fg_s),
                Span::styled("1", num_s),
            ]),
        ],
        PreviewLanguage::Cpp => vec![
            Line::from(vec![
                ln(1, false),
                Span::styled("// Modern C++20 Systems Engine (clangd + CMake)", cm_s),
            ]),
            Line::from(vec![
                ln(2, false),
                Span::styled("#include ", reg_s),
                Span::styled("<cstdint>", st_s),
            ]),
            Line::from(vec![ln(3, false)]),
            Line::from(vec![
                ln(4, false),
                Span::styled("template ", kw),
                Span::styled("<", op_s),
                Span::styled("typename ", kw),
                Span::styled("T", ty_s),
                Span::styled(">", op_s),
            ]),
            Line::from(vec![
                ln(5, false),
                Span::styled("constexpr ", kw),
                Span::styled("T ", ty_s),
                Span::styled("clamp_register", fn_s),
                Span::styled("(T val, T lo, T hi) ", fg_s),
                Span::styled("noexcept ", kw),
                Span::styled("{", fg_s),
            ]),
            Line::from(vec![
                ln(6, true),
                Span::styled(
                    "    return ",
                    Style::default().fg(p.keyword).bg(p.bg_highlight),
                ),
                Span::styled(
                    "(val < lo) ? lo : ((val > hi) ? hi : val);",
                    Style::default().fg(p.fg).bg(p.bg_highlight),
                ),
            ]),
            Line::from(vec![ln(7, false), Span::styled("}", fg_s)]),
        ],
    };

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(
            " NORMAL ",
            Style::default()
                .fg(p.bg)
                .bg(p.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "  main ",
            Style::default().fg(p.fg).bg(p.bg_highlight),
        ),
        Span::styled(
            format!(" 󰘚 ATmega644 [Debug|AVR] │ {} ", scheme_name),
            Style::default().fg(p.macro_reg).bg(p.bg_alt),
        ),
        Span::styled(
            " utf-8 │ 7:18 ",
            Style::default()
                .fg(p.bg)
                .bg(p.func)
                .add_modifier(Modifier::BOLD),
        ),
    ]));

    lines
}
