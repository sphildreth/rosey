mod app;
mod renderer;

use anyhow::Result;
use camino::Utf8PathBuf;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use rosey_core::plan_path;
use rosey_fs::{move_with_sidecars, Scanner};
use std::time::Duration;

use app::{AppState, IdentifiedItem, Screen, SortColumn, TransferItem, TransferState};

fn main() -> Result<()> {
    let source_path = std::env::args()
        .nth(1)
        .map(Utf8PathBuf::from)
        .unwrap_or_else(|| Utf8PathBuf::from("/dev/null"));

    let movies_target = std::env::args().nth(2).map(Utf8PathBuf::from);
    let tv_target = std::env::args().nth(3).map(Utf8PathBuf::from);

    let mut app = AppState::new(source_path, movies_target, tv_target);

    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    app.add_log("Rosey TUI started");

    let tick_rate = Duration::from_millis(100);
    let result = run_app(&mut terminal, &mut app, tick_rate);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    result?;
    Ok(())
}

fn run_app(
    terminal: &mut ratatui::Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut AppState,
    tick_rate: Duration,
) -> Result<()> {
    loop {
        terminal.draw(|f| renderer::render(f, app)).map_err(|e| anyhow::anyhow!("{e}"))?;

        if app.should_quit {
            return Ok(());
        }

        if event::poll(tick_rate)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }

                if app.show_confirmation {
                    match key.code {
                        KeyCode::Char('y') | KeyCode::Char('Y') => {
                            app.show_confirmation = false;
                            execute_move(app);
                            app.add_log("Move confirmed and executed");
                        }
                        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                            app.show_confirmation = false;
                            app.add_log("Move cancelled");
                        }
                        _ => {}
                    }
                    continue;
                }

                if app.filter_input_active {
                    match key.code {
                        KeyCode::Esc => {
                            app.filter_input_active = false;
                            app.search_query.clear();
                            app.rebuild_filtered();
                        }
                        KeyCode::Enter => {
                            app.filter_input_active = false;
                        }
                        KeyCode::Backspace => {
                            app.search_query.pop();
                            app.rebuild_filtered();
                        }
                        KeyCode::Char(c) => {
                            app.search_query.push(c);
                            app.rebuild_filtered();
                        }
                        _ => {}
                    }
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => {
                        app.should_quit = true;
                    }
                    KeyCode::Char('?') => {
                        app.current_screen = if app.current_screen == Screen::Help {
                            Screen::Dashboard
                        } else {
                            Screen::Help
                        };
                    }
                    KeyCode::Char('1') => app.current_screen = Screen::Dashboard,
                    KeyCode::Char('2') => app.current_screen = Screen::ScanResults,
                    KeyCode::Char('3') => app.current_screen = Screen::PlanPreview,
                    KeyCode::Char('4') => app.current_screen = Screen::TransferQueue,
                    KeyCode::Char('5') => app.current_screen = Screen::LogsRecovery,
                    KeyCode::Char('6') => app.current_screen = Screen::Settings,
                    KeyCode::Char('7') => app.current_screen = Screen::Help,
                    KeyCode::Char('s') if !app.scan_running => {
                        run_scan(app);
                    }
                    KeyCode::Char('p') if app.scan_complete => {
                        run_plan(app);
                    }
                    KeyCode::Char('m')
                        if !app.identified_items.is_empty() && app.transfer_queue.is_empty() =>
                    {
                        if app.dry_run {
                            execute_move(app);
                        } else {
                            app.show_confirmation = true;
                        }
                    }
                    KeyCode::Char('d') => {
                        app.dry_run = !app.dry_run;
                        app.add_log(format!("Dry-run mode: {}", app.dry_run));
                    }
                    KeyCode::Char('c') => {
                        app.next_conflict_policy();
                        app.add_log(format!("Conflict policy: {}", app.get_conflict_policy_name()));
                    }
                    KeyCode::Char('/') => {
                        app.filter_input_active = true;
                        app.current_screen = Screen::PlanPreview;
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') if app.confidence_threshold < 100 => {
                        app.confidence_threshold += 10;
                        app.add_log(format!("Confidence threshold: {}", app.confidence_threshold));
                    }
                    KeyCode::Char('-') if app.confidence_threshold >= 10 => {
                        app.confidence_threshold -= 10;
                        app.add_log(format!("Confidence threshold: {}", app.confidence_threshold));
                    }
                    KeyCode::Up if app.selected_index > 0 => {
                        app.selected_index -= 1;
                    }
                    KeyCode::Down if app.selected_index + 1 < app.filtered_items.len() => {
                        app.selected_index += 1;
                    }
                    KeyCode::Char('t') => {
                        app.toggle_sort(SortColumn::Title);
                    }
                    KeyCode::Char('y') => {
                        app.toggle_sort(SortColumn::Year);
                    }
                    KeyCode::Char('k') => {
                        app.toggle_sort(SortColumn::Kind);
                    }
                    KeyCode::Char('C') => {
                        app.toggle_sort(SortColumn::Confidence);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn run_scan(app: &mut AppState) {
    app.scan_running = true;
    app.scan_error = None;
    app.scan_complete = false;
    app.scan_results.clear();
    app.identified_items.clear();
    app.filtered_items.clear();
    app.add_log(format!("Scanning: {}", app.source_path));

    let source = app.source_path.clone();
    let max_workers = app.max_workers;
    let scanner = Scanner::new(max_workers, false);

    let results = scanner.scan(&source);
    let video_count = results.iter().filter(|r| r.is_video).count();

    app.scan_results = results;
    app.scan_running = false;
    app.scan_complete = true;
    app.add_log(format!("Scan complete: {} video files found", video_count));
    app.current_screen = Screen::ScanResults;
}

fn run_plan(app: &mut AppState) {
    app.identified_items.clear();
    app.add_log("Planning: identifying and scoring media files...");

    let movies_root = app.movies_target.as_ref().map(|p| p.as_str()).unwrap_or("");
    let tv_root = app.tv_target.as_ref().map(|p| p.as_str()).unwrap_or("");

    let mut items = Vec::new();

    for scan_result in &app.scan_results {
        if !scan_result.is_video || scan_result.error.is_some() {
            continue;
        }

        let item = rosey_core::identify_file(&scan_result.path);
        let score = rosey_core::score_identification(&item);

        if score.confidence < app.confidence_threshold {
            continue;
        }

        let destination = plan_path(&item, movies_root, tv_root);

        items.push(IdentifiedItem { media_item: item, score, destination });
    }

    app.identified_items = items;
    app.rebuild_filtered();
    app.selected_index = 0;
    app.add_log(format!("Planning complete: {} items identified", app.identified_items.len()));
    app.current_screen = Screen::PlanPreview;
}

fn execute_move(app: &mut AppState) {
    let items: Vec<IdentifiedItem> = app.identified_items.clone();
    let conflict_policy = app.conflict_policy;
    let dry_run = app.dry_run;

    app.transfer_queue = items
        .iter()
        .map(|item| TransferItem { item: item.clone(), state: TransferState::Pending, error: None })
        .collect();

    app.transfer_progress = (0, app.transfer_queue.len());
    app.transfer_running = true;

    let mut moved = 0;
    for i in 0..app.transfer_queue.len() {
        let result = move_with_sidecars(
            &app.transfer_queue[i].item.media_item,
            &app.transfer_queue[i].item.destination,
            conflict_policy,
            dry_run,
        );

        if result.success {
            moved += 1;
            app.transfer_queue[i].state =
                if dry_run { TransferState::Skipped } else { TransferState::Completed };
        } else {
            app.transfer_queue[i].state = TransferState::Failed;
            app.transfer_queue[i].error = result.errors.first().cloned();
        }

        app.transfer_progress = (i + 1, app.transfer_queue.len());
        app.add_log(format!(
            "[{}/{}] {} -> {}",
            i + 1,
            app.transfer_queue.len(),
            app.transfer_queue[i].item.media_item.source_path,
            app.transfer_queue[i].item.destination
        ));
    }

    app.transfer_running = false;
    app.add_log(format!("Move complete: {}/{} succeeded", moved, app.transfer_queue.len()));
    app.current_screen = Screen::TransferQueue;
}
