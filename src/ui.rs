use crate::app::{App, FilterTab, PreviewLanguage};
use crate::themes::{ThemeEntry, ThemePalette};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    let root = frame.area();

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header + Filter Tabs
            Constraint::Min(14),   // Main Split (Theme List | Live Code Preview)
            Constraint::Length(3), // Status / Keybindings Footer
        ])
        .split(root);

    draw_header(frame, app, vertical[0]);

    let main_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(vertical[1]);

    draw_left_panel(frame, app, main_split[0]);
    draw_right_preview(frame, app, main_split[1]);
    draw_footer(frame, app, vertical[2]);

    if app.show_help {
        draw_help_modal(frame, root);
    }
}

fn draw_header(frame: &mut Frame, app: &App, area: Rect) {
    let header_cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(42),
            Constraint::Percentage(33),
            Constraint::Percentage(25),
        ])
        .split(area);

    // 1. Title & Active Theme
    let title_line = Line::from(vec![
        Span::styled(
            " 🎨 LazyVim Theme Studio ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("│ Aktiv: "),
        Span::styled(
            format!(" {} ", app.active_colorscheme),
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    let title_block = Paragraph::new(title_line).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Neovim / LazyVim "),
    );
    frame.render_widget(title_block, header_cols[0]);

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
                .title(" Filter [Tab / 1-3] "),
        )
        .style(Style::default().fg(Color::Gray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        );
    frame.render_widget(tabs, header_cols[1]);

    // 3. Search Bar
    let search_border_color = if app.is_searching {
        Color::Yellow
    } else {
        Color::DarkGray
    };
    let search_text = if app.is_searching {
        format!("🔎 {}_", app.search_query)
    } else if app.search_query.is_empty() {
        "Drücke [/] zum Suchen...".to_string()
    } else {
        format!("🔎 {}", app.search_query)
    };
    let search = Paragraph::new(search_text)
        .style(Style::default().fg(if app.is_searching || !app.search_query.is_empty() {
            Color::White
        } else {
            Color::DarkGray
        }))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(search_border_color))
                .title(" Suche [/] "),
        );
    frame.render_widget(search, header_cols[2]);
}

fn draw_left_panel(frame: &mut Frame, app: &App, area: Rect) {
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(7)])
        .split(area);

    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .map(|&idx| {
            let t = &app.themes[idx];
            let is_active = t.colorscheme == app.active_colorscheme
                || (app.active_colorscheme == "tokyonight" && t.colorscheme == "tokyonight-moon");

            let mut spans = Vec::new();
            if is_active {
                spans.push(Span::styled(
                    "★ AKTIV ",
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ));
            } else if t.installed {
                spans.push(Span::styled(
                    "✓ Inst. ",
                    Style::default().fg(Color::Cyan),
                ));
            } else {
                spans.push(Span::styled(
                    "↓ Cloud ",
                    Style::default().fg(Color::DarkGray),
                ));
            }

            let mode_icon = if t.is_light { "☀ " } else { "☾ " };
            spans.push(Span::styled(
                mode_icon,
                Style::default().fg(if t.is_light {
                    Color::Yellow
                } else {
                    Color::Blue
                }),
            ));

            spans.push(Span::styled(
                format!("{:<24}", t.display_name),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(if is_active {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ));

            spans.push(Span::styled(
                format!(" [{}]", t.category_badge()),
                Style::default().fg(Color::LightMagenta),
            ));

            ListItem::new(Line::from(spans))
        })
        .collect();

    let list_title = format!(
        " Themes ({}/{}) — [↑/↓] Auswahl | [Enter] Aktivieren ",
        app.filtered_indices.len(),
        app.themes.len()
    );
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(list_title)
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(45, 55, 85))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut state = ListState::default();
    if !app.filtered_indices.is_empty() {
        state.select(Some(app.selected_pos));
    }
    frame.render_stateful_widget(list, left_chunks[0], &mut state);

    // Metadata Info Card
    let details_lines = if let Some(t) = app.current_theme() {
        let status_span = if t.installed {
            Span::styled(
                "Lokal installiert (~/.local/share/nvim/lazy)",
                Style::default().fg(Color::Green),
            )
        } else {
            Span::styled(
                "Wird bei [Enter] automatisch von GitHub geladen",
                Style::default().fg(Color::Yellow),
            )
        };
        vec![
            Line::from(vec![
                Span::styled("Colorscheme: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &t.colorscheme,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw("   "),
                Span::styled("GitHub: ", Style::default().fg(Color::Gray)),
                Span::styled(&t.repo, Style::default().fg(Color::LightBlue)),
            ]),
            Line::from(vec![
                Span::styled("Status:      ", Style::default().fg(Color::Gray)),
                status_span,
            ]),
            Line::from(vec![
                Span::styled("Info:        ", Style::default().fg(Color::Gray)),
                Span::styled(&t.description, Style::default().fg(Color::White)),
            ]),
        ]
    } else {
        vec![Line::from("Kein Theme ausgewählt.")]
    };

    let details = Paragraph::new(details_lines)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Theme-Details (dotfyle.com/neovim/colorscheme/top) "),
        );
    frame.render_widget(details, left_chunks[1]);
}

fn draw_right_preview(frame: &mut Frame, app: &App, area: Rect) {
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(10), Constraint::Length(5)])
        .split(area);

    if let Some(theme) = app.current_theme() {
        draw_code_preview(frame, theme, app.preview_lang, right_chunks[0]);
        draw_palette_swatches(frame, theme, right_chunks[1]);
    } else {
        let empty = Paragraph::new("Keine Vorschau verfügbar.")
            .block(Block::default().borders(Borders::ALL).title(" Live-Vorschau "));
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
        " Live Code-Vorschau: {} │ Sprache [l]: {} ",
        theme.display_name,
        lang.label()
    );

    let block = Block::default()
        .borders(Borders::ALL)
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
        ("Keyword", p.keyword),
        ("Func", p.func),
        ("String", p.string),
        ("Type", p.r#type),
        ("Macro/Reg", p.macro_reg),
        ("Number", p.number),
        ("Operator", p.operator),
        ("Comment", p.comment),
    ];

    let mut line1 = Vec::new();
    let mut line2 = Vec::new();
    for (name, color) in swatch_items {
        line1.push(Span::styled(" ███ ", Style::default().fg(color)));
        line1.push(Span::raw(" "));
        line2.push(Span::styled(
            format!("{:<5}", &name[..name.len().min(5)]),
            Style::default().fg(Color::Gray),
        ));
        line2.push(Span::raw(" "));
    }

    let swatches = Paragraph::new(vec![Line::from(line1), Line::from(line2)])
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" 24-Bit TrueColor Farbpalette "),
        );
    frame.render_widget(swatches, area);
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
    let status_color = if app.status_is_error {
        Color::Red
    } else {
        Color::Green
    };

    let footer = Paragraph::new(Line::from(vec![
        Span::styled(
            " STATUS: ",
            Style::default()
                .fg(Color::Black)
                .bg(status_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(&app.status_message, Style::default().fg(Color::White)),
        Span::raw("   │ "),
        Span::styled("[?] Hilfe", Style::default().fg(Color::Yellow)),
    ]))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, area);
}

fn draw_help_modal(frame: &mut Frame, area: Rect) {
    let popup_area = centered_rect(65, 65, area);
    frame.render_widget(Clear, popup_area);

    let lines = vec![
        Line::from(Span::styled(
            "Tastenkürzel — LazyVim Theme Studio",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from("  ↑ / ↓ oder j / k    Durch die Themes navigieren (mit Live-Vorschau)"),
        Line::from("  Enter oder a        Ausgewähltes Theme sicher installieren & aktivieren"),
        Line::from("  Tab oder 1 / 2 / 3  Zwischen 'Alle', 'Installiert' und 'Dotfyle Top' wechseln"),
        Line::from("  l                   Code-Sprache der Live-Vorschau wechseln (AVR C / Rust / Python / C++)"),
        Line::from("  /                   Themes nach Namen, Repo oder Beschreibung filtern"),
        Line::from("  r                   Rollback: Letztes Backup wiederherstellen"),
        Line::from("  ?                   Diese Hilfe ein-/ausblenden"),
        Line::from("  q oder Esc          Programm beenden"),
        Line::from(""),
        Line::from(Span::styled(
            "Sicherheitshinweis (Zero-Breakage Garantie):",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from("  • Dieses Tool schreibt ausschließlich in ~/.config/nvim/lua/plugins/colorscheme.lua."),
        Line::from("  • Vor jedem Speichern wird automatisch ein Backup (colorscheme.lua.bak) angelegt."),
        Line::from("  • Neue Dotfyle-Themes werden vor dem Aktivieren per git clone nach ~/.local/share/nvim/lazy geladen."),
        Line::from("  • Ein pcall()-Schutz fällt im Notfall automatisch auf 'tokyonight' zurück."),
    ];

    let modal = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Hilfe [? / Esc zum Schließen] "),
        );
    frame.render_widget(modal, popup_area);
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
                Span::styled("// Microchip Studio 7.0.2594 — ATmega644 AD/DA Wandler", cm_s),
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
                    "// Port A als Ausgang (R-2R Netzwerk)",
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
            Line::from(vec![ln(12, false)]),
            Line::from(vec![
                ln(13, false),
                Span::styled("int ", kw),
                Span::styled("main", fn_s),
                Span::styled("(", fg_s),
                Span::styled("void", kw),
                Span::styled(") {", fg_s),
            ]),
            Line::from(vec![
                ln(14, false),
                Span::styled("    ", fg_s),
                Span::styled("adc_init", fn_s),
                Span::styled("(", fg_s),
                Span::styled("0", num_s),
                Span::styled(");", fg_s),
            ]),
            Line::from(vec![
                ln(15, false),
                Span::styled("    return ", kw),
                Span::styled("0", num_s),
                Span::styled(";", fg_s),
            ]),
            Line::from(vec![ln(16, false), Span::styled("}", fg_s)]),
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
            Line::from(vec![ln(9, false)]),
            Line::from(vec![
                ln(10, false),
                Span::styled("pub fn ", kw),
                Span::styled("sample_adc", fn_s),
                Span::styled("(raw: ", fg_s),
                Span::styled("u16", ty_s),
                Span::styled(") -> ", op_s),
                Span::styled("SensorFrame ", ty_s),
                Span::styled("{", fg_s),
            ]),
            Line::from(vec![
                ln(11, false),
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
                ln(12, false),
                Span::styled("    println!", reg_s),
                Span::styled("(", fg_s),
                Span::styled("\"ADC = {voltage_mv:.2} mV\"", st_s),
                Span::styled(");", fg_s),
            ]),
            Line::from(vec![
                ln(13, false),
                Span::styled("    SensorFrame ", ty_s),
                Span::styled("{ id: ", fg_s),
                Span::styled("1", num_s),
                Span::styled(", voltage_mv }", fg_s),
            ]),
            Line::from(vec![ln(14, false), Span::styled("}", fg_s)]),
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
                Span::styled("compute_baud_prescaler", fn_s),
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
            Line::from(vec![
                ln(3, false),
                Span::styled("#include ", reg_s),
                Span::styled("<string_view>", st_s),
            ]),
            Line::from(vec![ln(4, false)]),
            Line::from(vec![
                ln(5, false),
                Span::styled("template ", kw),
                Span::styled("<", op_s),
                Span::styled("typename ", kw),
                Span::styled("T", ty_s),
                Span::styled(">", op_s),
            ]),
            Line::from(vec![
                ln(6, false),
                Span::styled("constexpr ", kw),
                Span::styled("T ", ty_s),
                Span::styled("clamp_register", fn_s),
                Span::styled("(T val, T lo, T hi) ", fg_s),
                Span::styled("noexcept ", kw),
                Span::styled("{", fg_s),
            ]),
            Line::from(vec![
                ln(7, true),
                Span::styled(
                    "    return ",
                    Style::default().fg(p.keyword).bg(p.bg_highlight),
                ),
                Span::styled(
                    "(val < lo) ? lo : ((val > hi) ? hi : val);",
                    Style::default().fg(p.fg).bg(p.bg_highlight),
                ),
            ]),
            Line::from(vec![ln(8, false), Span::styled("}", fg_s)]),
        ],
    };

    lines.push(Line::from(""));
    // Simulated Lualine Statusline inside the preview
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
