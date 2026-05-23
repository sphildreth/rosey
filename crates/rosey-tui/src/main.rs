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
use rosey_core::{
    clean_title_with_year, discover_companion_files, find_nfo_for_file, parse_nfo, plan_path,
    MediaItem, MediaKind, Score,
};
use rosey_fs::{move_with_sidecars, Scanner};
use std::collections::BTreeMap;
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

        let item = identify_file(&scan_result.path);
        let score = score_identification(&item);

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

fn identify_file(path: &camino::Utf8Path) -> MediaItem {
    let filename = path.file_name().unwrap_or(path.as_str());
    let folder_name = path.parent().and_then(|p| p.file_name()).unwrap_or("").to_string();

    let nfo_data = find_nfo_for_file(path).and_then(|p| parse_nfo(&p));
    let companions = discover_companion_files(path);

    let year = rosey_core::extract_year(filename).or_else(|| {
        if !folder_name.is_empty() {
            rosey_core::extract_year(&folder_name)
        } else {
            None
        }
    });

    let episode_info = rosey_core::extract_episode_info(filename, None).or_else(|| {
        if !folder_name.is_empty() {
            rosey_core::extract_episode_info(&folder_name, None)
        } else {
            None
        }
    });

    let date = rosey_core::extract_date(filename).map(|d| d.date);
    let part = rosey_core::extract_part(filename);

    let mut item = if let Some(ref nfo) = nfo_data {
        if nfo.season.is_some() || nfo.episode.is_some() {
            MediaItem {
                kind: MediaKind::Episode,
                source_path: path.to_path_buf(),
                title: nfo.title.clone(),
                year: nfo.year,
                season: nfo.season,
                episodes: nfo.episode.map(|e| vec![e]).unwrap_or_default(),
                part,
                date: date.clone(),
                sidecars: companions,
                nfo: {
                    let mut map = BTreeMap::new();
                    if let Some(id) = &nfo.tmdb_id {
                        map.insert("tmdbid".to_string(), Some(id.clone()));
                    }
                    if let Some(id) = &nfo.imdb_id {
                        map.insert("imdbid".to_string(), Some(id.clone()));
                    }
                    if let Some(id) = &nfo.tvdb_id {
                        map.insert("tvdbid".to_string(), Some(id.clone()));
                    }
                    if let Some(title) = &nfo.episode_title {
                        map.insert("episode_title".to_string(), Some(title.clone()));
                    }
                    map
                },
            }
        } else if nfo.tmdb_id.is_some() || nfo.imdb_id.is_some() {
            MediaItem {
                kind: MediaKind::Movie,
                source_path: path.to_path_buf(),
                title: nfo.title.clone(),
                year: nfo.year,
                season: None,
                episodes: Vec::new(),
                part,
                date: date.clone(),
                sidecars: companions,
                nfo: {
                    let mut map = BTreeMap::new();
                    if let Some(id) = &nfo.tmdb_id {
                        map.insert("tmdbid".to_string(), Some(id.clone()));
                    }
                    if let Some(id) = &nfo.imdb_id {
                        map.insert("imdbid".to_string(), Some(id.clone()));
                    }
                    map
                },
            }
        } else {
            MediaItem::unknown(path)
        }
    } else if let Some(ep) = episode_info {
        MediaItem {
            kind: MediaKind::Episode,
            source_path: path.to_path_buf(),
            title: {
                let cleaned = clean_title_with_year(filename, year);
                if cleaned.is_empty() {
                    Some(filename.to_string())
                } else {
                    Some(cleaned)
                }
            },
            year,
            season: Some(ep.season),
            episodes: ep.episodes,
            part,
            date: date.clone(),
            sidecars: companions,
            nfo: BTreeMap::new(),
        }
    } else if date.is_some() {
        MediaItem {
            kind: MediaKind::Episode,
            source_path: path.to_path_buf(),
            title: {
                let cleaned = clean_title_with_year(filename, year);
                if cleaned.is_empty() {
                    Some(filename.to_string())
                } else {
                    Some(cleaned)
                }
            },
            year,
            season: None,
            episodes: Vec::new(),
            part,
            date: date.clone(),
            sidecars: companions,
            nfo: BTreeMap::new(),
        }
    } else if year.is_some() || part.is_some() {
        MediaItem {
            kind: MediaKind::Movie,
            source_path: path.to_path_buf(),
            title: {
                let cleaned = clean_title_with_year(filename, year);
                if cleaned.is_empty() {
                    Some(filename.to_string())
                } else {
                    Some(cleaned)
                }
            },
            year,
            season: None,
            episodes: Vec::new(),
            part,
            date: date.clone(),
            sidecars: companions,
            nfo: BTreeMap::new(),
        }
    } else {
        MediaItem::unknown(path)
    };

    if item.title.is_none() && !folder_name.is_empty() {
        let folder_year = rosey_core::extract_year(&folder_name);
        item.title = Some(clean_title_with_year(&folder_name, folder_year));
    }

    item
}

fn score_identification(item: &MediaItem) -> Score {
    let mut confidence: u8 = 0;
    let mut reasons: Vec<String> = Vec::new();

    match item.kind {
        MediaKind::Movie => {
            if item.title.is_some() {
                confidence += 40;
                reasons.push("title found".to_string());
            }
            if item.year.is_some() {
                confidence += 30;
                reasons.push("year found".to_string());
            }
            if !item.nfo.is_empty() {
                confidence += 20;
                reasons.push("NFO data".to_string());
            }
            if item.part.is_some() {
                confidence += 10;
                reasons.push("part found".to_string());
            }
        }
        MediaKind::Episode => {
            if item.title.is_some() {
                confidence += 30;
                reasons.push("title found".to_string());
            }
            if item.season.is_some() && !item.episodes.is_empty() {
                confidence += 40;
                reasons.push("episode info found".to_string());
            }
            if item.date.is_some() {
                confidence += 40;
                reasons.push("date found".to_string());
            }
            if item.year.is_some() {
                confidence += 10;
                reasons.push("year found".to_string());
            }
            if !item.nfo.is_empty() {
                confidence += 20;
                reasons.push("NFO data".to_string());
            }
        }
        _ => {
            if item.title.is_some() {
                confidence += 10;
                reasons.push("title found".to_string());
            }
        }
    }

    confidence = confidence.min(100);
    Score { confidence, reasons }
}
