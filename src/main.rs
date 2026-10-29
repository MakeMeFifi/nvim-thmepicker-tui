mod app;
mod themes;
mod ui;

use anyhow::Result;
use app::{App, FilterTab, ListRow};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::env;
use std::io;
use std::time::Duration;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--list") {
        let mut app = App::new();
        if args.iter().any(|a| a == "--load-more") {
            app.load_more_themes();
        }
        println!("Active Neovim Colorscheme: {}", app.active_colorscheme);
        println!(
            "{:<5} {:<24} {:<22} {:<12} {:<28}",
            "RANK", "NAME", "COLORSCHEME", "STATUS", "REPO"
        );
        println!("{}", "-".repeat(95));
        for row in &app.rows {
            match row {
                ListRow::Theme(idx) => {
                    let t = &app.themes[*idx];
                    let rank = t
                        .dotfyle_rank
                        .map(|r| format!("#{:02}", r))
                        .unwrap_or_else(|| "LOC".to_string());
                    let status = if t.colorscheme == app.active_colorscheme {
                        "ACTIVE"
                    } else if t.installed {
                        "INSTALLED"
                    } else {
                        "DOTFYLE"
                    };
                    println!(
                        "{:<5} {:<24} {:<22} {:<12} {:<28}",
                        rank, t.display_name, t.colorscheme, status, t.repo
                    );
                }
                ListRow::LoadMoreButton => {
                    println!("      [ + Load More (+10 Themes from Dotfyle) ]");
                }
            }
        }
        return Ok(());
    }

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let run_result = run_app(&mut terminal, &mut app);

    // Always restore terminal cleanly even on panic/error
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    run_result
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // 1. Help modal mode
                if app.show_help {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('?') | KeyCode::Char('q') | KeyCode::Enter => {
                            app.show_help = false;
                        }
                        _ => {}
                    }
                    continue;
                }

                // 2. Search input mode
                if app.is_searching {
                    match key.code {
                        KeyCode::Esc | KeyCode::Enter => {
                            app.is_searching = false;
                        }
                        KeyCode::Backspace => {
                            app.search_query.pop();
                            app.recompute_filter();
                        }
                        KeyCode::Up => app.move_up(),
                        KeyCode::Down => app.move_down(),
                        KeyCode::Char(c) => {
                            app.search_query.push(c);
                            app.recompute_filter();
                        }
                        _ => {}
                    }
                    continue;
                }

                // 3. Normal navigation mode
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        if !app.search_query.is_empty() && key.code == KeyCode::Esc {
                            app.search_query.clear();
                            app.recompute_filter();
                        } else {
                            return Ok(());
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => app.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => app.move_down(),
                    KeyCode::Tab => {
                        app.tab = app.tab.next();
                        app.recompute_filter();
                    }
                    KeyCode::Char('1') => {
                        app.tab = FilterTab::All;
                        app.recompute_filter();
                    }
                    KeyCode::Char('2') => {
                        app.tab = FilterTab::Installed;
                        app.recompute_filter();
                    }
                    KeyCode::Char('3') => {
                        app.tab = FilterTab::DotfyleTop;
                        app.recompute_filter();
                    }
                    KeyCode::Char('l') | KeyCode::Char('L') => {
                        app.preview_lang = app.preview_lang.next();
                    }
                    KeyCode::Char('m') | KeyCode::Char('M') => {
                        app.status_message =
                            "Fetching +10 themes from dotfyle.com/neovim/colorscheme/top...".to_string();
                        terminal.draw(|f| ui::draw(f, app))?;
                        app.load_more_themes();
                    }
                    KeyCode::Char('/') => {
                        app.is_searching = true;
                    }
                    KeyCode::Char('?') => {
                        app.show_help = !app.show_help;
                    }
                    KeyCode::Enter | KeyCode::Char('a') => {
                        if matches!(app.current_row(), Some(ListRow::LoadMoreButton)) {
                            app.status_message =
                                "Fetching +10 themes from dotfyle.com/neovim/colorscheme/top...".to_string();
                            terminal.draw(|f| ui::draw(f, app))?;
                            app.load_more_themes();
                        } else {
                            app.status_message =
                                "Installing & activating selected theme...".to_string();
                            terminal.draw(|f| ui::draw(f, app))?;
                            if let Err(err) = app.apply_selected_theme() {
                                app.status_is_error = true;
                                app.status_message = format!("Error: {err}");
                            }
                        }
                    }
                    KeyCode::Char('r') => {
                        if let Err(err) = app.rollback_theme() {
                            app.status_is_error = true;
                            app.status_message = format!("Rollback error: {err}");
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
