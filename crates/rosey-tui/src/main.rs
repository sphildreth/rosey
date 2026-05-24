mod app;
mod renderer;
mod theme;

use anyhow::Result;
use camino::Utf8PathBuf;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use rosey_core::{
    config_path, load_config, plan_path, save_config, score_identification_result, IdentifyOptions,
    MediaKind,
};
use rosey_fs::{
    move_with_sidecars_journaled, scan_with_progress, OperationJournal, ScanOptions, ScanResult,
};
use rosey_metadata::{identify_file_with_metadata, ProviderManager};
use std::{
    fs,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::{Duration, Instant},
};

use app::{
    AppState, IdentifiedItem, ManualProviderResult, Screen, SortColumn, TransferItem, TransferState,
};

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
        journal_path: Option<Utf8PathBuf>,
    },
    ManualSearchComplete {
        results: Vec<ManualProviderResult>,
        elapsed: Duration,
    },
    ManualSearchFailed {
        message: String,
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

                if app.settings_edit.is_some() {
                    match key.code {
                        KeyCode::Esc => {
                            app.settings_edit = None;
                            app.add_log("Settings edit cancelled");
                        }
                        KeyCode::Enter => match app.apply_settings_edit() {
                            Ok(()) => {
                                app.add_log("Settings field updated");
                                app.set_operation(
                                    "Settings updated",
                                    "Press w or s on Settings to save to rosey.json.",
                                );
                            }
                            Err(err) => {
                                app.add_log(format!("Settings edit failed: {err}"));
                                app.set_operation("Settings edit failed", err);
                            }
                        },
                        KeyCode::Backspace => app.settings_edit_backspace(),
                        KeyCode::Char(c) => app.settings_edit_push_char(c),
                        _ => {}
                    }
                    continue;
                }

                if app.manual_edit.is_some() {
                    match key.code {
                        KeyCode::Esc => {
                            app.manual_edit = None;
                            app.manual_search_running = false;
                            app.add_log("Manual identification cancelled");
                        }
                        KeyCode::Enter => {
                            if app.manual_search_running {
                                app.set_manual_search_status(
                                    "Online search is still running. Wait for results or press Esc.",
                                );
                                continue;
                            }
                            if app.has_manual_provider_selection() {
                                app.apply_manual_provider_selection();
                                app.add_log("Provider identification applied");
                            } else {
                                app.apply_manual_edit();
                                app.add_log("Manual identification applied");
                            }
                            app.set_operation(
                                "Identification updated",
                                "Identification changes were applied.",
                            );
                        }
                        KeyCode::Tab => app.manual_next_field(),
                        KeyCode::F(5) => {
                            if let Some(rx) = start_manual_provider_search(app) {
                                worker_rx = Some(rx);
                            }
                        }
                        KeyCode::Up => app.manual_select_provider_result(-1),
                        KeyCode::Down => app.manual_select_provider_result(1),
                        KeyCode::Backspace => app.manual_backspace(),
                        KeyCode::Char(c) => app.manual_push_char(c),
                        _ => {}
                    }
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
                    KeyCode::Char('7') => app.current_screen = Screen::Doctor,
                    KeyCode::Char('8') => app.current_screen = Screen::Help,
                    KeyCode::Char('o') | KeyCode::Char('O')
                        if app.current_screen == Screen::Doctor && !app.is_busy() =>
                    {
                        app.refresh_doctor();
                        app.add_log("Doctor checks refreshed");
                    }
                    KeyCode::Up if app.current_screen == Screen::Doctor => {
                        app.doctor_scroll_up(1);
                    }
                    KeyCode::Down if app.current_screen == Screen::Doctor => {
                        app.doctor_scroll_down(1);
                    }
                    KeyCode::PageUp if app.current_screen == Screen::Doctor => {
                        app.doctor_scroll_up(8);
                    }
                    KeyCode::PageDown if app.current_screen == Screen::Doctor => {
                        app.doctor_scroll_down(8);
                    }
                    KeyCode::Char('w')
                    | KeyCode::Char('W')
                    | KeyCode::Char('s')
                    | KeyCode::Char('S')
                        if app.current_screen == Screen::Settings =>
                    {
                        save_settings(app);
                    }
                    KeyCode::Char('e') | KeyCode::Char('E')
                        if app.current_screen == Screen::Settings && !app.is_busy() =>
                    {
                        app.begin_settings_edit();
                    }
                    KeyCode::Char('s') if !app.is_busy() => {
                        worker_rx = start_scan(app);
                    }
                    KeyCode::Char('p') if app.scan_complete && !app.is_busy() => {
                        worker_rx = start_plan(app);
                    }
                    KeyCode::Char('m') if !app.identified_items.is_empty() && !app.is_busy() => {
                        if app.conflict_policy_ask && has_destination_conflicts(app) {
                            app.current_screen = Screen::TransferQueue;
                            app.set_operation(
                                "Conflict policy needed",
                                "Destination conflicts found. Press c to choose Skip, Replace, or Keep Both before moving.",
                            );
                            app.add_log(
                                "Move blocked: conflict policy is Ask and destination conflicts exist",
                            );
                        } else if app.dry_run {
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
                    KeyCode::Char('r')
                        if app.current_screen == Screen::LogsRecovery && !app.is_busy() =>
                    {
                        inspect_last_journal(app);
                    }
                    KeyCode::Char('x')
                        if app.current_screen == Screen::TransferQueue && !app.is_busy() =>
                    {
                        cleanup_after_move(app);
                    }
                    KeyCode::Char('/') => {
                        app.filter_input_active = true;
                        app.current_screen = Screen::PlanPreview;
                    }
                    KeyCode::Char('i')
                        if app.current_screen == Screen::PlanPreview
                            && !app.filtered_items.is_empty()
                            && !app.is_busy() =>
                    {
                        app.begin_manual_edit();
                    }
                    KeyCode::Char('+') | KeyCode::Char('=') if app.confidence_threshold < 100 => {
                        app.confidence_threshold += 10;
                        app.add_log(format!("Confidence threshold: {}", app.confidence_threshold));
                    }
                    KeyCode::Char('-') if app.confidence_threshold >= 10 => {
                        app.confidence_threshold -= 10;
                        app.add_log(format!("Confidence threshold: {}", app.confidence_threshold));
                    }
                    KeyCode::Up if app.current_screen == Screen::Settings => {
                        app.settings_previous_field();
                    }
                    KeyCode::Down if app.current_screen == Screen::Settings => {
                        app.settings_next_field();
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
    let config = app.config.clone();
    let movies_root = app.movies_target.as_ref().map(|p| p.to_string()).unwrap_or_default();
    let tv_root = app.tv_target.as_ref().map(|p| p.to_string()).unwrap_or_default();
    let threshold = app.confidence_threshold;
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let started = Instant::now();
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().ok();
        let mut items = Vec::new();
        let mut processed = 0;

        for scan_result in scan_results {
            if !scan_result.is_video || scan_result.error.is_some() {
                continue;
            }

            processed += 1;
            let ident = if let Some(runtime) = runtime.as_ref() {
                runtime.block_on(identify_file_with_metadata(
                    &scan_result.path,
                    &config,
                    IdentifyOptions { skip_duration: true },
                ))
            } else {
                rosey_core::identify_file_fast(&scan_result.path, &config)
            };
            let score = score_identification_result(&ident);
            let item = ident.item;

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
        let journal = OperationJournal::create_temp("rosey_move").ok();
        let journal_path = journal.as_ref().map(|journal| journal.path().to_path_buf());

        for (index, item) in items.into_iter().enumerate() {
            let result = move_with_sidecars_journaled(
                &item.media_item,
                &item.destination,
                conflict_policy,
                dry_run,
                journal.as_ref(),
            );
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

        if let Some(journal) = journal {
            journal.close();
        }

        let _ = tx.send(WorkerMessage::MoveComplete {
            succeeded,
            total,
            elapsed: started.elapsed(),
            journal_path,
        });
    });

    rx
}

fn start_manual_provider_search(app: &mut AppState) -> Option<Receiver<WorkerMessage>> {
    if app.manual_search_running {
        app.set_manual_search_status("Online search is already running.");
        return None;
    }

    if !app.config.identification.use_online_providers {
        app.set_manual_search_status("Online providers are disabled in config.");
        app.add_log("Manual identify search skipped: online providers disabled");
        return None;
    }

    if app.config.providers.tmdb_api_key.is_empty() {
        app.set_manual_search_status("TMDB API key is not configured.");
        app.add_log("Manual identify search skipped: TMDB API key missing");
        return None;
    }

    let edit = app.manual_edit.as_mut()?;
    let title = edit.title.trim().to_string();
    if title.is_empty() {
        edit.search_status = "Enter a title before searching.".to_string();
        return None;
    }

    let kind = edit.kind;
    let year = edit.year.trim().parse().ok();
    edit.provider_results.clear();
    edit.provider_index = 0;
    edit.search_status = format!("Searching online providers for {title}...");
    app.manual_search_running = true;
    app.set_operation("Searching providers", format!("Looking up {title}"));

    let config = app.config.clone();
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let started = Instant::now();
        let Ok(runtime) = tokio::runtime::Builder::new_current_thread().enable_all().build() else {
            let _ = tx.send(WorkerMessage::ManualSearchFailed {
                message: "Failed to start async provider runtime".to_string(),
            });
            return;
        };

        let results = runtime.block_on(async move {
            let Ok(mut manager) = ProviderManager::new(
                metadata_cache_dir(),
                config.providers.cache_ttl_days,
                config.identification.use_online_providers,
            ) else {
                return Err("Failed to initialize metadata provider cache".to_string());
            };

            manager.configure_tmdb(
                config.providers.tmdb_api_key.clone(),
                config.providers.tmdb_language.clone(),
                config.providers.tmdb_region.clone(),
            );

            let values = if kind == MediaKind::Episode {
                manager.search_tv(&title, year, true).await
            } else {
                manager.search_movie(&title, year, true).await
            };

            Ok(values
                .into_iter()
                .take(10)
                .filter_map(|value| provider_result_from_value(value, kind))
                .collect::<Vec<_>>())
        });

        match results {
            Ok(results) => {
                let _ = tx.send(WorkerMessage::ManualSearchComplete {
                    results,
                    elapsed: started.elapsed(),
                });
            }
            Err(message) => {
                let _ = tx.send(WorkerMessage::ManualSearchFailed { message });
            }
        }
    });

    Some(rx)
}

fn has_destination_conflicts(app: &AppState) -> bool {
    app.identified_items.iter().any(|item| item.destination.exists())
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
                    app.manual_search_running = false;
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
        WorkerMessage::MoveComplete { succeeded, total, elapsed, journal_path } => {
            app.transfer_running = false;
            app.transfer_success_count = succeeded;
            app.transfer_progress = (total, total);
            app.last_journal_path = journal_path.clone();
            app.set_operation(
                if app.dry_run { "Dry-run complete" } else { "Move complete" },
                format!("{succeeded}/{total} succeeded in {}", format_duration(elapsed)),
            );
            app.add_log(format!(
                "Move complete: {succeeded}/{total} succeeded in {}",
                format_duration(elapsed)
            ));
            if let Some(path) = journal_path {
                app.add_log(format!("Move journal: {path}"));
            }
            app.current_screen = Screen::TransferQueue;
            true
        }
        WorkerMessage::ManualSearchComplete { results, elapsed } => {
            let count = results.len();
            app.manual_search_running = false;
            app.set_manual_search_results(results);
            app.set_operation(
                "Provider search complete",
                format!("{count} result(s) in {}", format_duration(elapsed)),
            );
            app.add_log(format!(
                "Provider search complete: {count} result(s) in {}",
                format_duration(elapsed)
            ));
            true
        }
        WorkerMessage::ManualSearchFailed { message } => {
            app.manual_search_running = false;
            app.set_manual_search_status(message.clone());
            app.set_operation("Provider search failed", message.clone());
            app.add_log(format!("Provider search failed: {message}"));
            true
        }
    }
}

fn inspect_last_journal(app: &mut AppState) {
    let Some(path) = app.last_journal_path.clone() else {
        app.recovery_summary = "No move journal is available yet.".to_string();
        app.set_operation("Recovery", app.recovery_summary.clone());
        app.add_log(app.recovery_summary.clone());
        return;
    };

    match OperationJournal::read_entries(&path) {
        Ok(entries) => {
            let incomplete = OperationJournal::find_incomplete_transfers(&entries);
            app.recovery_summary = format!(
                "{} journal entries inspected; {} incomplete transfer(s) found in {}",
                entries.len(),
                incomplete.len(),
                path
            );
            app.set_operation("Recovery inspected", app.recovery_summary.clone());
            app.add_log(app.recovery_summary.clone());
        }
        Err(err) => {
            app.recovery_summary = format!("Failed to read move journal {path}: {err}");
            app.set_operation("Recovery failed", app.recovery_summary.clone());
            app.add_log(app.recovery_summary.clone());
        }
    }
}

fn cleanup_after_move(app: &mut AppState) {
    if app.dry_run {
        app.set_operation("Cleanup skipped", "Cleanup is disabled in dry-run mode.");
        app.add_log("Cleanup skipped: dry-run mode");
        return;
    }

    let mut directories = std::collections::BTreeSet::new();
    for transfer in &app.transfer_queue {
        if transfer.state == TransferState::Completed {
            if let Some(parent) = transfer.item.media_item.source_path.parent() {
                directories.insert(parent.to_path_buf());
            }
        }
    }

    if directories.is_empty() {
        app.set_operation("Cleanup skipped", "No completed live moves to clean up.");
        app.add_log("Cleanup skipped: no completed live moves");
        return;
    }

    let mut deleted_files = 0;
    let mut removed_dirs = 0;
    let patterns = app.config.behavior.auto_delete_patterns.clone();

    for directory in &directories {
        deleted_files += delete_matching_files(directory, &patterns);
    }

    for directory in directories.iter().rev() {
        removed_dirs += remove_empty_directories(directory, &app.source_path);
    }

    let summary = format!(
        "Cleanup complete: {deleted_files} pattern file(s) deleted, {removed_dirs} empty directories removed"
    );
    app.set_operation("Cleanup complete", summary.clone());
    app.add_log(summary);
}

fn delete_matching_files(directory: &camino::Utf8Path, patterns: &[String]) -> usize {
    let Ok(entries) = fs::read_dir(directory.as_std_path()) else {
        return 0;
    };
    let mut count = 0;

    for entry in entries.flatten() {
        let Ok(path) = Utf8PathBuf::from_path_buf(entry.path()) else {
            continue;
        };
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name() else {
            continue;
        };
        if patterns.iter().any(|pattern| pattern_matches(pattern, name))
            && fs::remove_file(path.as_std_path()).is_ok()
        {
            count += 1;
        }
    }

    count
}

fn pattern_matches(pattern: &str, name: &str) -> bool {
    let pattern = pattern.to_ascii_lowercase();
    let name = name.to_ascii_lowercase();

    if let Some(ext) = pattern.strip_prefix("*.") {
        return name.rsplit_once('.').map(|(_, candidate)| candidate == ext).unwrap_or(false);
    }

    pattern == name
}

fn remove_empty_directories(directory: &camino::Utf8Path, stop_at: &camino::Utf8Path) -> usize {
    let mut count = 0;
    let mut current = directory.to_path_buf();

    loop {
        if current == stop_at || !current.starts_with(stop_at) {
            break;
        }
        let is_empty = fs::read_dir(current.as_std_path())
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false);
        if !is_empty {
            break;
        }
        if fs::remove_dir(current.as_std_path()).is_err() {
            break;
        }
        count += 1;
        let Some(parent) = current.parent() else {
            break;
        };
        current = parent.to_path_buf();
    }

    count
}

fn save_settings(app: &mut AppState) {
    app.sync_settings_to_config();
    let path = config_path();

    match save_config(&app.config) {
        Ok(()) => {
            let detail = format!("Saved settings to {}", path.display());
            app.set_operation("Settings saved", detail.clone());
            app.add_log(detail);
        }
        Err(err) => {
            let detail = format!("Failed to save settings to {}: {}", path.display(), err);
            app.set_operation("Settings save failed", detail.clone());
            app.add_log(detail);
        }
    }
}

fn metadata_cache_dir() -> std::path::PathBuf {
    config_path()
        .parent()
        .map(|path| path.join("cache"))
        .unwrap_or_else(|| std::path::PathBuf::from(".").join("cache"))
}

fn provider_result_from_value(
    value: serde_json::Value,
    requested_kind: MediaKind,
) -> Option<ManualProviderResult> {
    let id = value.get("id")?.as_u64()?.to_string();
    let title_key = if requested_kind == MediaKind::Episode { "name" } else { "title" };
    let date_key =
        if requested_kind == MediaKind::Episode { "first_air_date" } else { "release_date" };
    let title = value
        .get(title_key)
        .and_then(|title| title.as_str())
        .filter(|title| !title.is_empty())?
        .to_string();
    let year = value
        .get(date_key)
        .and_then(|date| date.as_str())
        .and_then(|date| date.get(0..4))
        .and_then(|year| year.parse().ok());

    Some(ManualProviderResult { id, kind: requested_kind, title, year })
}

fn format_duration(duration: Duration) -> String {
    let millis = duration.as_millis();
    if millis < 1_000 {
        format!("{millis}ms")
    } else {
        format!("{:.1}s", millis as f64 / 1_000.0)
    }
}
