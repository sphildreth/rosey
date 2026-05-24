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
use rosey_core::{load_config, plan_path};
use rosey_fs::{move_with_sidecars, scan_with_progress, ScanOptions, ScanResult};
use std::{
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::{Duration, Instant},
};

use app::{AppState, IdentifiedItem, Screen, SortColumn, TransferItem, TransferState};

fn main() -> Result<()> {
    let config = load_config();
    let mut args = std::env::args().skip(1);
    let source_arg = args.next().map(Utf8PathBuf::from);
    let movies_arg = args.next().map(Utf8PathBuf::from);
    let tv_arg = args.next().map(Utf8PathBuf::from);

    let source_path =
        resolve_path(source_arg, &config.paths.source).unwrap_or_else(|| Utf8PathBuf::from(""));
    let movies_target = resolve_path(movies_arg, &config.paths.movies);
    let tv_target = resolve_path(tv_arg, &config.paths.tv);

    let mut app = AppState::new(&config, source_path, movies_target, tv_target);

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

fn resolve_path(value: Option<Utf8PathBuf>, config_value: &str) -> Option<Utf8PathBuf> {
    value.or_else(|| {
        if config_value.is_empty() {
            None
        } else {
            Some(Utf8PathBuf::from(config_value))
        }
    })
}

enum WorkerMessage {
    ScanProgress {
        result: ScanResult,
        scanned: usize,
        video_count: usize,
        error_count: usize,
    },
    ScanComplete {
        results: Vec<ScanResult>,
        video_count: usize,
        error_count: usize,
        elapsed: Duration,
    },
    PlanProgress {
        processed: usize,
        total: usize,
        kept: usize,
        current: Utf8PathBuf,
    },
    PlanComplete {
        items: Vec<IdentifiedItem>,
        processed: usize,
        elapsed: Duration,
    },
    MoveProgress {
        index: usize,
        total: usize,
        state: TransferState,
        error: Option<String>,
        source: Utf8PathBuf,
        destination: Utf8PathBuf,
    },
    MoveComplete {
        succeeded: usize,
        total: usize,
        elapsed: Duration,
    },
}

fn run_app(
    terminal: &mut ratatui::Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut AppState,
    tick_rate: Duration,
) -> Result<()> {
    let mut worker_rx: Option<Receiver<WorkerMessage>> = None;

    loop {
        app.tick();
        drain_worker_messages(app, &mut worker_rx);
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
                            worker_rx = Some(start_move(app));
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
                        if app.is_busy() {
                            app.add_log("Finish or wait for the active operation before quitting");
                            app.set_operation(
                                "Operation running",
                                "Quit is disabled while Rosey is scanning, planning, or moving.",
                            );
                        } else {
                            app.should_quit = true;
                        }
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
                    KeyCode::Char('s') if !app.is_busy() => {
                        worker_rx = start_scan(app);
                    }
                    KeyCode::Char('p') if app.scan_complete && !app.is_busy() => {
                        worker_rx = start_plan(app);
                    }
                    KeyCode::Char('m') if !app.identified_items.is_empty() && !app.is_busy() => {
                        if app.dry_run {
                            worker_rx = Some(start_move(app));
                        } else {
                            app.show_confirmation = true;
                        }
                    }
                    KeyCode::Char('d') => {
                        if app.is_busy() {
                            app.add_log("Cannot change dry-run mode during an active operation");
                        } else {
                            app.dry_run = !app.dry_run;
                            app.add_log(format!("Dry-run mode: {}", app.dry_run));
                        }
                    }
                    KeyCode::Char('c') => {
                        if app.is_busy() {
                            app.add_log("Cannot change conflict policy during an active operation");
                        } else {
                            app.next_conflict_policy();
                            app.add_log(format!(
                                "Conflict policy: {}",
                                app.get_conflict_policy_name()
                            ));
                        }
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

fn start_scan(app: &mut AppState) -> Option<Receiver<WorkerMessage>> {
    if app.source_path.as_str().is_empty() {
        let message =
            "No source path configured. Pass a source path or set paths.source in config."
                .to_string();
        app.scan_error = Some(message.clone());
        app.set_operation("Scan failed", &message);
        app.add_log(message);
        app.current_screen = Screen::ScanResults;
        return None;
    }

    if !app.source_path.exists() {
        let message = format!("Source path does not exist: {}", app.source_path);
        app.scan_error = Some(message.clone());
        app.set_operation("Scan failed", &message);
        app.add_log(message);
        app.current_screen = Screen::ScanResults;
        return None;
    }

    app.scan_running = true;
    app.scan_error = None;
    app.scan_complete = false;
    app.scan_progress = (0, 0);
    app.scan_results.clear();
    app.identified_items.clear();
    app.filtered_items.clear();
    app.transfer_queue.clear();
    app.transfer_progress = (0, 0);
    app.transfer_success_count = 0;
    app.add_log(format!("Scanning: {}", app.source_path));
    app.set_operation("Scanning", format!("Walking {}", app.source_path));
    app.current_screen = Screen::ScanResults;

    let source = app.source_path.clone();
    let max_workers = app.max_workers;
    let follow_symlinks = app.follow_symlinks;
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let started = Instant::now();
        let progress_tx = tx.clone();
        let mut scanned = 0;
        let mut video_count = 0;
        let mut error_count = 0;
        let results = scan_with_progress(
            &source,
            ScanOptions { follow_symlinks, max_depth: None, max_workers },
            |result| {
                scanned += 1;
                if result.is_video {
                    video_count += 1;
                }
                if result.error.is_some() {
                    error_count += 1;
                }
                let _ = progress_tx.send(WorkerMessage::ScanProgress {
                    result: result.clone(),
                    scanned,
                    video_count,
                    error_count,
                });
            },
        );
        let _ = tx.send(WorkerMessage::ScanComplete {
            results,
            video_count,
            error_count,
            elapsed: started.elapsed(),
        });
    });

    Some(rx)
}

fn start_plan(app: &mut AppState) -> Option<Receiver<WorkerMessage>> {
    let total = app.scan_results.iter().filter(|r| r.is_video && r.error.is_none()).count();
    if total == 0 {
        app.add_log("No scanned video files to plan");
        app.set_operation("Planning skipped", "Run a scan with video results first.");
        return None;
    }

    app.plan_running = true;
    app.plan_progress = (0, total);
    app.identified_items.clear();
    app.filtered_items.clear();
    app.add_log("Planning: identifying and scoring media files...");
    app.set_operation("Planning", format!("0/{total} files processed"));
    app.current_screen = Screen::PlanPreview;

    let scan_results = app.scan_results.clone();
    let movies_root = app.movies_target.as_ref().map(|p| p.to_string()).unwrap_or_default();
    let tv_root = app.tv_target.as_ref().map(|p| p.to_string()).unwrap_or_default();
    let threshold = app.confidence_threshold;
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let started = Instant::now();
        let mut items = Vec::new();
        let mut processed = 0;

        for scan_result in scan_results {
            if !scan_result.is_video || scan_result.error.is_some() {
                continue;
            }

            processed += 1;
            let item = rosey_core::identify_file(&scan_result.path);
            let score = rosey_core::score_identification(&item);

            if score.confidence >= threshold {
                let destination = plan_path(&item, movies_root.as_str(), tv_root.as_str());
                items.push(IdentifiedItem { media_item: item, score, destination });
            }

            let _ = tx.send(WorkerMessage::PlanProgress {
                processed,
                total,
                kept: items.len(),
                current: scan_result.path,
            });
        }

        let _ =
            tx.send(WorkerMessage::PlanComplete { items, processed, elapsed: started.elapsed() });
    });

    Some(rx)
}

fn start_move(app: &mut AppState) -> Receiver<WorkerMessage> {
    let items: Vec<IdentifiedItem> = app.identified_items.clone();
    let conflict_policy = app.conflict_policy;
    let dry_run = app.dry_run;
    let total = items.len();

    app.transfer_queue = items
        .iter()
        .map(|item| TransferItem { item: item.clone(), state: TransferState::Pending, error: None })
        .collect();

    app.transfer_progress = (0, app.transfer_queue.len());
    app.transfer_success_count = 0;
    app.transfer_running = true;
    app.current_screen = Screen::TransferQueue;
    app.set_operation(
        if dry_run { "Dry-run transfer" } else { "Moving files" },
        format!("0/{total} items processed"),
    );

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let started = Instant::now();
        let mut succeeded = 0;

        for (index, item) in items.into_iter().enumerate() {
            let result =
                move_with_sidecars(&item.media_item, &item.destination, conflict_policy, dry_run);
            let state = if result.success {
                succeeded += 1;
                if dry_run {
                    TransferState::WouldMove
                } else if !result.skipped.is_empty()
                    && result.moved.is_empty()
                    && result.replaced.is_empty()
                    && result.kept_both.is_empty()
                {
                    TransferState::Skipped
                } else {
                    TransferState::Completed
                }
            } else {
                TransferState::Failed
            };

            let _ = tx.send(WorkerMessage::MoveProgress {
                index,
                total,
                state,
                error: result.errors.first().cloned(),
                source: item.media_item.source_path,
                destination: item.destination,
            });
        }

        let _ =
            tx.send(WorkerMessage::MoveComplete { succeeded, total, elapsed: started.elapsed() });
    });

    rx
}

fn drain_worker_messages(app: &mut AppState, worker_rx: &mut Option<Receiver<WorkerMessage>>) {
    let Some(rx) = worker_rx.take() else {
        return;
    };

    let mut finished = false;
    loop {
        match rx.try_recv() {
            Ok(message) => {
                if handle_worker_message(app, message) {
                    finished = true;
                }
            }
            Err(TryRecvError::Empty) => break,
            Err(TryRecvError::Disconnected) => {
                if app.is_busy() {
                    app.scan_running = false;
                    app.plan_running = false;
                    app.transfer_running = false;
                    app.scan_error = Some("Background operation ended unexpectedly".to_string());
                    app.set_operation(
                        "Operation interrupted",
                        "The background worker exited before reporting completion.",
                    );
                    app.add_log("Background operation ended unexpectedly");
                }
                finished = true;
                break;
            }
        }
    }

    if !finished {
        *worker_rx = Some(rx);
    }
}

fn handle_worker_message(app: &mut AppState, message: WorkerMessage) -> bool {
    match message {
        WorkerMessage::ScanProgress { result, scanned, video_count, error_count } => {
            app.scan_progress = (scanned, 0);
            app.scan_results.push(result.clone());
            app.set_operation(
                "Scanning",
                format!("{video_count} video files, {error_count} errors found: {}", result.path),
            );
            false
        }
        WorkerMessage::ScanComplete { results, video_count, error_count, elapsed } => {
            let total = results.len();
            app.scan_results = results;
            app.scan_running = false;
            app.scan_complete = true;
            app.scan_progress = (total, total);
            app.set_operation(
                "Scan complete",
                format!(
                    "{video_count} video files, {error_count} errors in {}",
                    format_duration(elapsed)
                ),
            );
            app.add_log(format!(
                "Scan complete: {video_count} video files found in {}",
                format_duration(elapsed)
            ));
            app.current_screen = Screen::ScanResults;
            true
        }
        WorkerMessage::PlanProgress { processed, total, kept, current } => {
            app.plan_progress = (processed, total);
            app.set_operation(
                "Planning",
                format!("{processed}/{total} files processed, {kept} kept: {current}"),
            );
            false
        }
        WorkerMessage::PlanComplete { items, processed, elapsed } => {
            app.identified_items = items;
            app.rebuild_filtered();
            app.selected_index = 0;
            app.plan_running = false;
            app.plan_progress = (processed, processed);
            app.set_operation(
                "Planning complete",
                format!(
                    "{} items identified from {processed} files in {}",
                    app.identified_items.len(),
                    format_duration(elapsed)
                ),
            );
            app.add_log(format!(
                "Planning complete: {} items identified in {}",
                app.identified_items.len(),
                format_duration(elapsed)
            ));
            app.current_screen = Screen::PlanPreview;
            true
        }
        WorkerMessage::MoveProgress { index, total, state, error, source, destination } => {
            if let Some(item) = app.transfer_queue.get_mut(index) {
                item.state = state;
                item.error = error.clone();
            }
            if matches!(
                state,
                TransferState::Completed | TransferState::WouldMove | TransferState::Skipped
            ) {
                app.transfer_success_count += 1;
            }
            app.transfer_progress = (index + 1, total);
            app.set_operation(
                if app.dry_run { "Dry-run transfer" } else { "Moving files" },
                format!("{}/{} processed: {source} -> {destination}", index + 1, total),
            );
            if let Some(error) = error {
                app.add_log(format!(
                    "[{}/{}] failed: {} -> {} ({})",
                    index + 1,
                    total,
                    source,
                    destination,
                    error
                ));
            } else {
                app.add_log(format!("[{}/{}] {} -> {}", index + 1, total, source, destination));
            }
            false
        }
        WorkerMessage::MoveComplete { succeeded, total, elapsed } => {
            app.transfer_running = false;
            app.transfer_success_count = succeeded;
            app.transfer_progress = (total, total);
            app.set_operation(
                if app.dry_run { "Dry-run complete" } else { "Move complete" },
                format!("{succeeded}/{total} succeeded in {}", format_duration(elapsed)),
            );
            app.add_log(format!(
                "Move complete: {succeeded}/{total} succeeded in {}",
                format_duration(elapsed)
            ));
            app.current_screen = Screen::TransferQueue;
            true
        }
    }
}

fn format_duration(duration: Duration) -> String {
    let millis = duration.as_millis();
    if millis < 1_000 {
        format!("{millis}ms")
    } else {
        format!("{:.1}s", millis as f64 / 1_000.0)
    }
}
