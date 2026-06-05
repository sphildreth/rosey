use crate::app::{
    AppState, ManualField, PendingDelete, Screen, SettingsField, SortColumn, SortDirection,
    TransferState,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Table, Wrap,
    },
    Frame,
};
use rosey_core::{ConfidenceBand, DoctorStatus, MediaKind};

pub fn render(frame: &mut Frame, app: &AppState) {
    if app.pending_delete.is_some() {
        render_delete_confirmation_dialog(frame, app);
        return;
    }

    if app.show_confirmation {
        render_confirmation_dialog(frame, app);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(2),
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_tabs(frame, app, chunks[1]);

    match app.current_screen {
        Screen::Dashboard => render_dashboard(frame, app, chunks[2]),
        Screen::ScanResults => render_scan_results(frame, app, chunks[2]),
        Screen::PlanPreview => render_plan_preview(frame, app, chunks[2]),
        Screen::TransferQueue => render_transfer_queue(frame, app, chunks[2]),
        Screen::LogsRecovery => render_logs(frame, app, chunks[2]),
        Screen::Settings => render_settings(frame, app, chunks[2]),
        Screen::Doctor => render_doctor(frame, app, chunks[2]),
        Screen::Help => render_help(frame, app, chunks[2]),
    }

    render_status_bar(frame, app, chunks[3]);

    if app.manual_edit.is_some() {
        render_manual_identify_dialog(frame, app);
    }

    if app.settings_edit.is_some() {
        render_settings_edit_dialog(frame, app);
    }
}

fn render_header(frame: &mut Frame, app: &AppState, area: Rect) {
    let title = Paragraph::new(Text::from(vec![
        Line::from(vec![Span::styled("Rosey", app.theme.title)]),
        Line::from(vec![Span::raw("Media File Organizer")]),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(app.theme.title_border))
    .centered();

    frame.render_widget(title, area);
}

fn render_tabs(frame: &mut Frame, app: &AppState, area: Rect) {
    let tabs: Vec<Span> = Screen::all()
        .iter()
        .map(|s| {
            let label = format!(" {}:{} ", s.shortcut(), s.title());
            if *s == app.current_screen {
                Span::styled(label, app.theme.tab_active)
            } else {
                Span::styled(label, app.theme.tab_inactive)
            }
        })
        .collect();

    let tab_line = Line::from(tabs);
    let tab_widget = Paragraph::new(tab_line).block(Block::default());
    frame.render_widget(tab_widget, area);
}

fn render_dashboard(frame: &mut Frame, app: &AppState, area: Rect) {
    let theme = &app.theme;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    let info_lines = vec![
        Line::from(vec![
            Span::styled("Source:         ", theme.dim),
            Span::styled(format!("{}", app.source_path), theme.strong),
        ]),
        Line::from(vec![
            Span::styled("Movies Target:  ", theme.dim),
            Span::styled(
                app.movies_target.as_ref().map(|p| p.as_str()).unwrap_or("(not set)"),
                theme.strong,
            ),
        ]),
        Line::from(vec![
            Span::styled("TV Target:      ", theme.dim),
            Span::styled(
                app.tv_target.as_ref().map(|p| p.as_str()).unwrap_or("(not set)"),
                theme.strong,
            ),
        ]),
        Line::from(vec![
            Span::styled("Mode:           ", theme.dim),
            Span::styled(
                if app.dry_run { "DRY-RUN" } else { "LIVE" },
                if app.dry_run { theme.warn } else { theme.error },
            ),
        ]),
        Line::from(vec![
            Span::styled("Conflict:       ", theme.dim),
            Span::styled(app.get_conflict_policy_name(), theme.strong),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("s", theme.key),
            Span::raw(" scan    "),
            Span::styled("p", theme.key),
            Span::raw(" plan    "),
            Span::styled("m", theme.key),
            Span::raw(" move    "),
            Span::styled("?", theme.key),
            Span::raw(" help    "),
            Span::styled("q", theme.key),
            Span::raw(" quit"),
        ]),
    ];

    let info = Paragraph::new(info_lines)
        .block(Block::default().title(" Overview ").borders(Borders::ALL));

    frame.render_widget(info, chunks[0]);
    render_activity(frame, app, chunks[1]);

    let high = app.confidence_thresholds.green.max(app.confidence_thresholds.yellow);
    let low = app.confidence_thresholds.green.min(app.confidence_thresholds.yellow);
    let stats_lines = vec![
        Line::from(vec![
            Span::styled("Scanned:  ", theme.dim),
            Span::raw(format!(
                "{} video files",
                app.scan_results.iter().filter(|r| r.is_video).count()
            )),
        ]),
        Line::from(vec![
            Span::styled("Planned:  ", theme.dim),
            Span::raw(format!("{} items", app.identified_items.len())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Green:    ", theme.ok),
            Span::raw(format!(
                "{} ",
                app.identified_items.iter().filter(|i| i.score.confidence >= high).count()
            )),
            Span::styled("Yellow:   ", theme.warn),
            Span::raw(format!(
                "{} ",
                app.identified_items
                    .iter()
                    .filter(|i| (low..high).contains(&i.score.confidence))
                    .count()
            )),
            Span::styled("Red:      ", theme.error),
            Span::raw(format!(
                "{}",
                app.identified_items.iter().filter(|i| i.score.confidence < low).count()
            )),
        ]),
        Line::from(vec![
            Span::styled("Transferred: ", theme.dim),
            Span::raw(format!("{}/{}", app.transfer_progress.0, app.transfer_queue.len())),
        ]),
    ];

    let stats =
        Paragraph::new(stats_lines).block(Block::default().title(" Status ").borders(Borders::ALL));

    frame.render_widget(stats, chunks[2]);
}

fn render_activity(frame: &mut Frame, app: &AppState, area: Rect) {
    let ratio = active_progress_ratio(app).unwrap_or_else(|| {
        if app.is_busy() {
            ((app.activity_tick % 20) + 1) as f64 / 20.0
        } else {
            0.0
        }
    });

    let marker = if app.is_busy() { spinner(app.activity_tick) } else { "OK" };
    let label = truncate_to_width(
        &format!("{marker} {} - {}", app.operation_status, app.operation_detail),
        area.width.saturating_sub(4) as usize,
    );
    let style = if app.operation_status.to_lowercase().contains("failed") {
        app.theme.error
    } else if app.is_busy() {
        app.theme.gauge
    } else {
        app.theme.ok
    };

    let gauge = Gauge::default()
        .gauge_style(style)
        .label(label)
        .ratio(ratio)
        .block(Block::default().title(" Activity ").borders(Borders::ALL));

    frame.render_widget(gauge, area);
}

fn render_scan_results(frame: &mut Frame, app: &AppState, area: Rect) {
    if app.scan_error.is_some() {
        let error =
            Paragraph::new(format!("Scan error: {}", app.scan_error.as_deref().unwrap_or("")))
                .style(app.theme.error)
                .block(Block::default().borders(Borders::ALL).border_style(app.theme.border_error));

        frame.render_widget(error, area);
        return;
    }

    let (detail_area, table_area) = if app.scan_running {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Length(5), Constraint::Min(1)])
            .split(area);
        render_activity(frame, app, chunks[0]);
        (chunks[1], chunks[2])
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Min(1)])
            .split(area);
        (chunks[0], chunks[1])
    };
    render_scan_detail(frame, app, detail_area);

    let rows: Vec<Row> = app
        .scan_results
        .iter()
        .enumerate()
        .map(|(index, r)| {
            let row_style =
                if index == app.selected_scan_index { app.theme.selected } else { Style::new() };
            Row::new(vec![
                Cell::from(r.path.as_str()),
                Cell::from(if r.is_video { "video" } else { "other" }),
                Cell::from(rosey_fs::format_bytes(r.size_bytes)),
                Cell::from(r.error.as_deref().unwrap_or("")),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Percentage(50),
        Constraint::Percentage(10),
        Constraint::Percentage(15),
        Constraint::Percentage(25),
    ];

    let table = Table::new(rows, widths)
        .header(Row::new(vec!["Path", "Type", "Size", "Error"]).style(app.theme.header))
        .block(
            Block::default().title(" Scan Results (i=identify selected) ").borders(Borders::ALL),
        );

    frame.render_widget(table, table_area);
}

fn render_scan_detail(frame: &mut Frame, app: &AppState, area: Rect) {
    let lines = if let Some(result) = app.scan_results.get(app.selected_scan_index) {
        let identify_hint = if result.is_video && result.error.is_none() {
            "press i to identify/search providers and add or update the Move Plan"
        } else {
            "select a video row to identify it"
        };
        vec![
            Line::from(vec![Span::styled("Selected scan result", app.theme.header)]),
            Line::from(vec![
                Span::styled("File: ", app.theme.dim),
                Span::raw(result.path.as_str()),
            ]),
            Line::from(vec![Span::styled("Identify: ", app.theme.dim), Span::raw(identify_hint)]),
            Line::from(vec![
                Span::styled("Delete: ", app.theme.dim),
                Span::raw("press Del to delete this file's containing directory"),
            ]),
        ]
    } else {
        vec![
            Line::from(vec![Span::styled("Scan Results", app.theme.header)]),
            Line::from("Press s to scan the source path."),
            Line::from(vec![
                Span::styled("Identify: ", app.theme.dim),
                Span::raw("after scanning, select a video and press i"),
            ]),
            Line::from(vec![
                Span::styled("Delete: ", app.theme.dim),
                Span::raw("after scanning, select a video and press Del"),
            ]),
        ]
    };

    let detail = Paragraph::new(lines)
        .block(Block::default().title(" Identify Source ").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(detail, area);
}

fn render_plan_preview(frame: &mut Frame, app: &AppState, area: Rect) {
    let constraints = if app.plan_running {
        vec![
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Min(1),
        ]
    } else {
        vec![Constraint::Length(3), Constraint::Length(7), Constraint::Min(1)]
    };

    let chunks =
        Layout::default().direction(Direction::Vertical).constraints(constraints).split(area);
    let (detail_area, table_area) = if app.plan_running {
        render_activity(frame, app, chunks[1]);
        (chunks[2], chunks[3])
    } else {
        (chunks[1], chunks[2])
    };

    let mode_hint = if app.show_group_mode {
        if app.filter_input_active {
            format!("TV Shows | Filter: {}▌", app.search_query)
        } else {
            "TV Shows | G=flat view  Enter=expand  ↑↓=select".to_string()
        }
    } else if app.filter_input_active {
        format!("Filter: {}▌", app.search_query)
    } else {
        format!("Filter: {} (press / to search)", app.search_query)
    };

    let search = Paragraph::new(mode_hint).block(Block::default().borders(Borders::ALL));
    frame.render_widget(search, chunks[0]);

    if app.show_group_mode {
        render_show_group_detail(frame, app, detail_area);
    } else {
        render_plan_detail(frame, app, detail_area);
    }

    if let Some(message) = plan_empty_message(app) {
        let empty = Paragraph::new(message)
            .block(Block::default().title(" Move Plan ").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        frame.render_widget(empty, table_area);
        return;
    }

    if app.show_group_mode {
        render_show_group_table(frame, app, table_area);
        return;
    }

    let sort_hint = format!(
        "Sort: {} {}  t=title  y=year  c=confidence  k=kind  d=dest",
        match app.sort_column {
            SortColumn::Title => "Title",
            SortColumn::Year => "Year",
            SortColumn::Confidence => "Conf",
            SortColumn::Kind => "Kind",
            SortColumn::Destination => "Dest",
        },
        match app.sort_direction {
            SortDirection::Asc => "▲",
            SortDirection::Desc => "▼",
        }
    );

    let (visible_start, visible_end) =
        plan_visible_window(app.selected_index, app.filtered_items.len(), table_area.height);

    let rows: Vec<Row> = app
        .filtered_items
        .iter()
        .enumerate()
        .skip(visible_start)
        .take(visible_end.saturating_sub(visible_start))
        .map(|(idx, &item_idx)| {
            let item = &app.identified_items[item_idx];
            let style = match configured_confidence_band(app, item.score.confidence) {
                ConfidenceBand::Green => app.theme.ok,
                ConfidenceBand::Yellow => app.theme.warn,
                ConfidenceBand::Red => app.theme.error,
            };

            let row_style =
                if idx == app.selected_index { app.theme.selected } else { Style::new() };

            let kind_str = match item.media_item.kind {
                MediaKind::Movie => "Movie",
                MediaKind::Episode => "Episode",
                MediaKind::Show => "Show",
                MediaKind::Unknown => "?",
            };

            Row::new(vec![
                Cell::from(format!("{:3}%", item.score.confidence)).style(style),
                duplicate_indicator_cell(app, item),
                Cell::from(kind_str),
                Cell::from(item.media_item.title.as_deref().unwrap_or("?")),
                Cell::from(
                    item.media_item.year.map(|y| y.to_string()).unwrap_or_else(|| "-".to_string()),
                ),
                Cell::from(item.media_item.source_path.as_str()),
                Cell::from(item.destination.as_str()),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(5),
        Constraint::Length(5),
        Constraint::Length(7),
        Constraint::Percentage(18),
        Constraint::Length(6),
        Constraint::Percentage(32),
        Constraint::Percentage(32),
    ];

    let range_hint = if app.filtered_items.len() > visible_end.saturating_sub(visible_start) {
        format!("rows {}-{} of {}", visible_start + 1, visible_end, app.filtered_items.len())
    } else {
        format!("{} items", app.filtered_items.len())
    };

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec!["Conf", "Dup", "Kind", "Title", "Year", "Source", "Destination"])
                .style(app.theme.header),
        )
        .block(
            Block::default()
                .title(format!(" Move Plan ({range_hint}) - {sort_hint} "))
                .borders(Borders::ALL),
        );

    frame.render_widget(table, table_area);
}

fn plan_visible_window(selected_index: usize, total: usize, table_height: u16) -> (usize, usize) {
    if total == 0 {
        return (0, 0);
    }

    let visible_rows = usize::from(table_height.saturating_sub(3)).max(1).min(total);
    let selected_index = selected_index.min(total.saturating_sub(1));
    let mut start = selected_index.saturating_add(1).saturating_sub(visible_rows);

    if start + visible_rows > total {
        start = total.saturating_sub(visible_rows);
    }

    (start, start + visible_rows)
}

fn render_plan_detail(frame: &mut Frame, app: &AppState, area: Rect) {
    let lines = if let Some(&item_index) = app.filtered_items.get(app.selected_index) {
        let item = &app.identified_items[item_index];
        let mut lines = vec![
            Line::from(vec![
                Span::styled("Selected move: ", app.theme.header),
                Span::styled(item.media_item.title.as_deref().unwrap_or("?"), app.theme.strong),
                Span::raw(format!("  ({:3}%)", item.score.confidence)),
            ]),
            Line::from(vec![
                Span::styled("From: ", app.theme.dim),
                Span::raw(item.media_item.source_path.as_str()),
            ]),
            Line::from(vec![
                Span::styled("To:   ", app.theme.dim),
                Span::raw(item.destination.as_str()),
            ]),
        ];
        if let Some(line) = duplicate_detail_line(app, item) {
            lines.push(line);
        }
        lines.extend([Line::from(vec![
            Span::styled("Move: ", app.theme.dim),
            Span::raw("press m to execute this plan; Del removes this item from the plan"),
        ])]);
        lines
    } else {
        vec![
            Line::from(vec![Span::styled("Move Plan", app.theme.header)]),
            Line::from(vec![Span::raw(plan_state_summary(app))]),
            Line::from(vec![
                Span::styled("Keys: ", app.theme.dim),
                Span::raw("s scan, p plan, m move, / filter"),
            ]),
        ]
    };

    let detail = Paragraph::new(lines)
        .block(Block::default().title(" Source -> Destination ").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(detail, area);
}

fn render_show_group_detail(frame: &mut Frame, app: &AppState, area: Rect) {
    let lines = if let Some(show) = app.tv_shows.get(app.selected_show_index) {
        let expanded = app.expanded_show_index == Some(app.selected_show_index);
        let mut lines = vec![
            Line::from(vec![
                Span::styled("TV Show: ", app.theme.header),
                Span::styled(&show.title, app.theme.strong),
                Span::raw(if let Some(year) = show.year {
                    format!(" ({year})")
                } else {
                    String::new()
                }),
                Span::raw(format!("  ({:3}%)", show.confidence)),
            ]),
            Line::from(vec![
                Span::styled("Seasons: ", app.theme.dim),
                Span::raw(format!("{}, {} episode(s)", show.seasons.len(), show.total_episodes())),
            ]),
        ];
        if let Some(tmdb_id) = &show.tmdb_id {
            lines.push(Line::from(vec![
                Span::styled("TMDB: ", app.theme.dim),
                Span::raw(tmdb_id.as_str()),
            ]));
        }
        if let Some(tvdb_id) = &show.tvdb_id {
            lines.push(Line::from(vec![
                Span::styled("TVDB: ", app.theme.dim),
                Span::raw(tvdb_id.as_str()),
            ]));
        }
        lines.push(Line::from(vec![
            Span::styled("Assets: ", app.theme.dim),
            Span::raw(format!(
                "{} show-level, {} season-level",
                show.show_assets.len(),
                show.seasons.iter().map(|s| s.season_assets.len()).sum::<usize>()
            )),
        ]));
        if expanded {
            for season in &show.seasons {
                lines.push(Line::from(vec![
                    Span::styled(format!("  Season {}: ", season.season_number), app.theme.strong),
                    Span::raw(format!("{} episode(s)", season.episodes.len())),
                ]));
            }
        }
        lines.push(Line::from(vec![
            Span::styled(if expanded { "Collapse: " } else { "Expand: " }, app.theme.dim),
            Span::styled("Enter", app.theme.key),
            Span::raw(if expanded { " to collapse" } else { " to expand seasons" }),
            Span::raw("   "),
            Span::styled("G", app.theme.key),
            Span::raw(" flat view"),
        ]));
        lines
    } else if app.tv_shows.is_empty() {
        vec![
            Line::from(vec![Span::styled("TV Show Groups", app.theme.header)]),
            Line::from("No TV shows detected. Press G to return to flat item view."),
        ]
    } else {
        vec![
            Line::from(vec![Span::styled("TV Show Groups", app.theme.header)]),
            Line::from(vec![Span::raw(plan_state_summary(app))]),
            Line::from(vec![
                Span::styled("Keys: ", app.theme.dim),
                Span::raw("↑↓ select show, Enter expand, G flat view"),
            ]),
        ]
    };

    let detail = Paragraph::new(lines)
        .block(Block::default().title(" TV Show Detail ").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    frame.render_widget(detail, area);
}

fn render_show_group_table(frame: &mut Frame, app: &AppState, area: Rect) {
    if app.tv_shows.is_empty() {
        let empty = Paragraph::new("No TV show groups found in the move plan.")
            .block(Block::default().title(" TV Shows ").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        frame.render_widget(empty, area);
        return;
    }

    let (visible_start, visible_end) =
        plan_visible_window(app.selected_show_index, app.tv_shows.len(), area.height);

    let expanded = app.expanded_show_index;

    let mut rows = Vec::new();
    for (idx, show) in app.tv_shows.iter().enumerate() {
        if idx < visible_start || idx >= visible_end {
            continue;
        }
        let is_selected = idx == app.selected_show_index;
        let row_style = if is_selected { app.theme.selected } else { Style::new() };
        let style = match configured_confidence_band(app, show.confidence) {
            ConfidenceBand::Green => app.theme.ok,
            ConfidenceBand::Yellow => app.theme.warn,
            ConfidenceBand::Red => app.theme.error,
        };
        let year_str = show.year.map(|y| y.to_string()).unwrap_or_else(|| "-".to_string());
        let ids = format_provider_ids(show);
        let dest = show
            .seasons
            .first()
            .and_then(|s| s.episodes.first())
            .map(|e| e.destination.as_str())
            .unwrap_or("?");

        rows.push(
            Row::new(vec![
                Cell::from(format!("{:3}%", show.confidence)).style(style),
                Cell::from("Show"),
                Cell::from(show.title.as_str()),
                Cell::from(year_str),
                Cell::from(ids),
                Cell::from(dest),
            ])
            .style(row_style),
        );

        if expanded == Some(idx) {
            for season in &show.seasons {
                let season_label = format!("  S{:02}", season.season_number);
                let ep_count = format!("{} ep(s)", season.episodes.len());
                rows.push(
                    Row::new(vec![
                        Cell::from(""),
                        Cell::from(season_label),
                        Cell::from(ep_count),
                        Cell::from(""),
                        Cell::from(""),
                        Cell::from(""),
                    ])
                    .style(app.theme.dim),
                );
            }
        }
    }

    let total_eps: usize = app.tv_shows.iter().map(|s| s.total_episodes()).sum();
    let total_assets: usize = app.tv_shows.iter().map(|s| s.show_assets.len()).sum::<usize>();
    let range_hint = if app.tv_shows.len() > visible_end.saturating_sub(visible_start) {
        format!(
            "rows {}-{} of {} shows, {} ep(s), {} asset(s)",
            visible_start + 1,
            visible_end,
            app.tv_shows.len(),
            total_eps,
            total_assets,
        )
    } else {
        format!("{} show(s), {} ep(s), {} asset(s)", app.tv_shows.len(), total_eps, total_assets,)
    };

    let widths = [
        Constraint::Length(5),
        Constraint::Length(7),
        Constraint::Percentage(25),
        Constraint::Length(6),
        Constraint::Percentage(20),
        Constraint::Percentage(40),
    ];

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec!["Conf", "Kind", "Title", "Year", "IDs", "Destination"])
                .style(app.theme.header),
        )
        .block(Block::default().title(format!(" TV Shows ({range_hint}) ")).borders(Borders::ALL));

    frame.render_widget(table, area);
}

fn format_provider_ids(show: &rosey_core::TvShow) -> String {
    let mut parts = Vec::new();
    if let Some(id) = &show.tmdb_id {
        parts.push(format!("tmdb:{}", id));
    }
    if let Some(id) = &show.tvdb_id {
        parts.push(format!("tvdb:{}", id));
    }
    if let Some(id) = &show.imdb_id {
        parts.push(format!("imdb:{}", id));
    }
    if parts.is_empty() {
        "-".to_string()
    } else {
        parts.join(", ")
    }
}

fn duplicate_indicator_cell<'a>(app: &AppState, item: &crate::app::IdentifiedItem) -> Cell<'a> {
    match item.duplicate_indicator() {
        Some("FILE") => Cell::from("FILE").style(app.theme.error),
        Some("DIR") => Cell::from("DIR").style(app.theme.warn),
        _ => Cell::from(""),
    }
}

fn duplicate_detail_line<'a>(
    app: &AppState,
    item: &crate::app::IdentifiedItem,
) -> Option<Line<'a>> {
    if item.destination_file_exists() {
        return Some(Line::from(vec![
            Span::styled("Duplicate: ", app.theme.warn),
            Span::styled("destination file already exists", app.theme.error),
        ]));
    }

    if item.destination_directory_exists() {
        let directory = item.destination_directory().map(|path| path.as_str()).unwrap_or("");
        return Some(Line::from(vec![
            Span::styled("Duplicate: ", app.theme.warn),
            Span::raw("likely - destination directory already exists: "),
            Span::styled(directory.to_string(), app.theme.dim),
        ]));
    }

    None
}

fn plan_empty_message(app: &AppState) -> Option<String> {
    if app.plan_running {
        return Some("Building the move plan. Identified files will appear here.".to_string());
    }

    if app.identified_items.is_empty() {
        let scanned_videos = app.scan_results.iter().filter(|result| result.is_video).count();
        if scanned_videos == 0 {
            return Some(
                "No scanned video files yet. Press s to scan the source path.".to_string(),
            );
        }

        if app.plan_progress.1 == 0 {
            return Some(format!(
                "Scan complete: {scanned_videos} video file(s) found. Press p to build the move plan."
            ));
        }

        return Some(format!(
            "Plan complete: 0 of {} scanned video file(s) met the yellow confidence threshold ({}%). Lower the threshold in Settings or press +/- here, then press p again.",
            app.plan_progress.1,
            app.confidence_threshold
        ));
    }

    if app.filtered_items.is_empty() {
        return Some(format!(
            "No planned items match filter '{}'. Press Esc to clear the filter.",
            app.search_query
        ));
    }

    None
}

fn plan_state_summary(app: &AppState) -> String {
    let scanned_videos = app.scan_results.iter().filter(|result| result.is_video).count();
    if app.plan_running {
        return format!(
            "Planning in progress: {}/{} scanned video file(s) processed.",
            app.plan_progress.0, app.plan_progress.1
        );
    }
    if app.identified_items.is_empty() && app.plan_progress.1 == 0 {
        return format!("{scanned_videos} scanned video file(s). Press p to build the move plan.");
    }
    format!(
        "{} planned move(s) from {} scanned video file(s).",
        app.identified_items.len(),
        scanned_videos
    )
}

fn render_transfer_queue(frame: &mut Frame, app: &AppState, area: Rect) {
    if app.transfer_queue.is_empty() {
        let empty = Paragraph::new(
            "No items in transfer queue.\n\nRun a plan, then press 'm' to preview or move planned items.",
        )
        .block(Block::default().borders(Borders::ALL))
        .centered();
        frame.render_widget(empty, area);
        return;
    }

    let progress = if app.transfer_queue.is_empty() {
        0.0
    } else {
        app.transfer_progress.0 as f64 / app.transfer_progress.1 as f64
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    let gauge = Gauge::default()
        .gauge_style(app.theme.gauge)
        .label(format!("Progress: {}/{}", app.transfer_progress.0, app.transfer_progress.1))
        .ratio(progress);

    frame.render_widget(gauge, chunks[0]);

    let rows: Vec<Row> = app
        .transfer_queue
        .iter()
        .map(|t| {
            let (state_str, state_style) = match t.state {
                TransferState::Pending => ("PENDING", app.theme.dim),
                TransferState::InProgress => ("MOVING...", app.theme.warn),
                TransferState::WouldMove => ("WOULD MOVE", app.theme.ok),
                TransferState::Completed => ("DONE", app.theme.ok),
                TransferState::Failed => ("FAILED", app.theme.error),
                TransferState::Skipped => ("SKIPPED", app.theme.warn),
            };

            Row::new(vec![
                Cell::from(state_str).style(state_style),
                Cell::from(t.item.media_item.title.as_deref().unwrap_or("?")),
                Cell::from(t.item.destination.as_str()),
                Cell::from(t.error.as_deref().unwrap_or("")),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(11),
        Constraint::Percentage(30),
        Constraint::Percentage(50),
        Constraint::Percentage(20),
    ];

    let table = Table::new(rows, widths)
        .header(Row::new(vec!["Status", "Title", "Destination", "Error"]).style(app.theme.header))
        .block(
            Block::default()
                .title(format!(" Transfer Queue ({}) ", app.transfer_queue.len()))
                .borders(Borders::ALL),
        );

    frame.render_widget(table, chunks[1]);
}

fn render_logs(frame: &mut Frame, app: &AppState, area: Rect) {
    let theme = &app.theme;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(1)])
        .split(area);

    let recovery = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Journal:  ", theme.dim),
            Span::raw(app.last_journal_path.as_ref().map(|p| p.as_str()).unwrap_or("(none)")),
        ]),
        Line::from(vec![
            Span::styled("Recovery: ", theme.dim),
            Span::raw(app.recovery_summary.as_str()),
        ]),
        Line::from(vec![Span::styled("r", theme.key), Span::raw(" inspect last move journal")]),
    ])
    .block(Block::default().title(" Recovery ").borders(Borders::ALL))
    .wrap(Wrap { trim: true });

    frame.render_widget(recovery, chunks[0]);

    let items: Vec<ListItem> =
        app.log_messages.iter().rev().take(50).map(|msg| ListItem::new(Span::raw(msg))).collect();

    let list = List::new(items).block(
        Block::default().title(format!(" Log ({}) ", app.log_messages.len())).borders(Borders::ALL),
    );

    frame.render_widget(list, chunks[1]);
}

fn render_settings(frame: &mut Frame, app: &AppState, area: Rect) {
    let mut settings_lines = Vec::new();
    settings_lines.push(Line::from(vec![
        Span::styled("Up/Down ", app.theme.key),
        Span::raw("select   "),
        Span::styled("e ", app.theme.key),
        Span::raw("edit   "),
        Span::styled("w/s ", app.theme.key),
        Span::raw("save"),
    ]));
    settings_lines.push(Line::from(""));

    for (index, field) in SettingsField::all().iter().enumerate() {
        let selected = index == app.selected_settings_index;
        let marker = if selected { ">" } else { " " };
        let label_style = if selected { app.theme.strong } else { app.theme.dim };
        let value = app.settings_value(*field);
        settings_lines.push(Line::from(vec![
            Span::styled(format!("{marker} {:<22}", field.label()), label_style),
            Span::raw(truncate_to_width(&value, area.width.saturating_sub(26) as usize)),
        ]));
    }

    settings_lines.push(Line::from(""));
    settings_lines.push(Line::from(vec![
        Span::styled("Status: ", app.theme.dim),
        Span::raw(app.operation_status.as_str()),
    ]));

    let settings = Paragraph::new(settings_lines)
        .block(Block::default().title(" Settings ").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(settings, area);
}

fn render_settings_edit_dialog(frame: &mut Frame, app: &AppState) {
    let Some(edit) = &app.settings_edit else {
        return;
    };

    let area = centered_rect(68, 24, frame.area());
    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(format!("Edit {}", edit.field.label()), app.theme.header)]),
        Line::from(""),
        Line::from(vec![Span::styled("Value: ", app.theme.dim), Span::raw(edit.value.as_str())]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", app.theme.key),
            Span::raw(" apply   "),
            Span::styled("Esc", app.theme.key),
            Span::raw(" cancel   "),
            Span::styled("Backspace", app.theme.key),
            Span::raw(" delete"),
        ]),
    ];

    let dialog = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Setting ")
                .borders(Borders::ALL)
                .border_type(BorderType::Double),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, frame.area());
    frame.render_widget(dialog, area);
}

fn render_doctor(frame: &mut Frame, app: &AppState, area: Rect) {
    let report = &app.doctor_report;
    let theme = &app.theme;
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Overall: ", theme.dim),
            Span::styled(
                doctor_status_label(report.overall),
                doctor_status_style(report.overall, app),
            ),
            Span::raw(format!("  {} errors, {} warnings", report.errors(), report.warnings())),
        ]),
        Line::from(vec![Span::styled("Config:  ", theme.dim), Span::raw(&report.config_path)]),
        Line::from(vec![
            Span::styled("Keys:    ", theme.dim),
            Span::raw("o refresh, Up/Down scroll, PgUp/PgDn page"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("Attention", theme.header)]),
    ];

    let issues: Vec<_> =
        report.checks.iter().filter(|check| check.status != DoctorStatus::Ok).collect();
    if issues.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("[OK] ", theme.ok),
            Span::raw("No warnings or errors."),
        ]));
    } else {
        for check in &issues {
            lines.push(doctor_check_line(check, app));
            if let Some(detail) = &check.detail {
                lines.push(Line::from(vec![Span::raw("      "), Span::styled(detail, theme.dim)]));
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("All Checks", theme.header)]));
    for check in &report.checks {
        lines.push(doctor_check_line(check, app));
        if let Some(detail) = &check.detail {
            lines.push(Line::from(vec![Span::raw("      "), Span::styled(detail, theme.dim)]));
        }
    }

    let scroll = app.doctor_scroll.min(lines.len().saturating_sub(1)) as u16;
    let doctor = Paragraph::new(lines)
        .block(Block::default().title(" Doctor ").borders(Borders::ALL))
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(doctor, area);
}

fn doctor_check_line<'a>(check: &'a rosey_core::DoctorCheck, app: &AppState) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("[{}] ", doctor_status_label(check.status)),
            doctor_status_style(check.status, app),
        ),
        Span::styled(format!("{}: ", check.name), app.theme.strong),
        Span::raw(&check.message),
    ])
}

fn doctor_status_label(status: DoctorStatus) -> &'static str {
    match status {
        DoctorStatus::Ok => "OK",
        DoctorStatus::Warn => "WARN",
        DoctorStatus::Error => "ERROR",
    }
}

fn doctor_status_style(status: DoctorStatus, app: &AppState) -> Style {
    match status {
        DoctorStatus::Ok => app.theme.ok,
        DoctorStatus::Warn => app.theme.warn,
        DoctorStatus::Error => app.theme.error,
    }
}

fn render_help(frame: &mut Frame, app: &AppState, area: Rect) {
    let help_lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("Keyboard Shortcuts", app.theme.header)]),
        Line::from(""),
        Line::from(vec![Span::styled("1-8 ", app.theme.key), Span::raw("Switch screens")]),
        Line::from(vec![Span::styled("s  ", app.theme.key), Span::raw("Scan source directory")]),
        Line::from(vec![
            Span::styled("p  ", app.theme.key),
            Span::raw("Plan; scans first when no scan is complete"),
        ]),
        Line::from(vec![
            Span::styled("m  ", app.theme.key),
            Span::raw("Execute move (requires confirmation in live mode)"),
        ]),
        Line::from(vec![Span::styled("d  ", app.theme.key), Span::raw("Toggle dry-run mode")]),
        Line::from(vec![
            Span::styled("c  ", app.theme.key),
            Span::raw("Cycle conflict policy (Skip → Replace → Keep Both)"),
        ]),
        Line::from(vec![
            Span::styled("/  ", app.theme.key),
            Span::raw("Search/filter in Plan Preview"),
        ]),
        Line::from(vec![
            Span::styled("i  ", app.theme.key),
            Span::raw("Identify selected scan result or planned move"),
        ]),
        Line::from(vec![
            Span::styled("Del", app.theme.key),
            Span::raw("Remove selected planned move or delete selected scan directory"),
        ]),
        Line::from(vec![
            Span::styled("F5 ", app.theme.key),
            Span::raw("Search providers from manual identify"),
        ]),
        Line::from(vec![
            Span::styled("r  ", app.theme.key),
            Span::raw("Inspect last move journal on Logs / Recovery"),
        ]),
        Line::from(vec![
            Span::styled("x  ", app.theme.key),
            Span::raw("Clean moved source folders on Transfer Queue"),
        ]),
        Line::from(vec![Span::styled("o  ", app.theme.key), Span::raw("Refresh Doctor checks")]),
        Line::from(vec![Span::styled("Esc", app.theme.key), Span::raw("Clear filter")]),
        Line::from(vec![Span::styled("↑↓ ", app.theme.key), Span::raw("Navigate items")]),
        Line::from(vec![
            Span::styled("+/-", app.theme.key),
            Span::raw("Adjust confidence threshold"),
        ]),
        Line::from(vec![
            Span::styled("G  ", app.theme.key),
            Span::raw("Toggle TV show group view"),
        ]),
        Line::from(vec![
            Span::styled("e  ", app.theme.key),
            Span::raw("Edit selected Settings field"),
        ]),
        Line::from(vec![
            Span::styled("y/n", app.theme.key),
            Span::raw("Confirm/cancel in confirmation dialog"),
        ]),
        Line::from(vec![Span::styled("q  ", app.theme.key), Span::raw("Quit")]),
        Line::from(vec![Span::styled("?  ", app.theme.key), Span::raw("Toggle help")]),
    ];

    let help =
        Paragraph::new(help_lines).block(Block::default().title(" Help ").borders(Borders::ALL));

    frame.render_widget(help, area);
}

fn render_delete_confirmation_dialog(frame: &mut Frame, app: &AppState) {
    let area = frame.area();
    let dialog_area = centered_rect(72, 46, area);

    let lines = match app.pending_delete.as_ref() {
        Some(PendingDelete::PlanItem { item_index }) => {
            let item = app.identified_items.get(*item_index);
            let title = item
                .and_then(|item| item.media_item.title.as_deref())
                .unwrap_or("selected planned move");
            let source = item.map(|item| item.media_item.source_path.as_str()).unwrap_or("");
            let destination = item.map(|item| item.destination.as_str()).unwrap_or("");
            vec![
                Line::from(vec![Span::styled("Confirm Plan Removal", app.theme.danger_bold)]),
                Line::from(""),
                Line::from(vec![
                    Span::styled("Action: ", app.theme.dim),
                    Span::raw("Remove this item from the Move Plan."),
                ]),
                Line::from(vec![Span::styled("Title:  ", app.theme.dim), Span::raw(title)]),
                Line::from(vec![Span::styled("Source: ", app.theme.dim), Span::raw(source)]),
                Line::from(vec![Span::styled("Dest:   ", app.theme.dim), Span::raw(destination)]),
                Line::from(""),
                Line::from(vec![Span::styled("y removes only from the plan.", app.theme.ok)]),
                Line::from(vec![Span::styled(
                    "Del deletes the source file from disk and removes it from the plan.",
                    app.theme.error,
                )]),
                Line::from(""),
                Line::from(vec![Span::styled(
                    "y = remove    Del = delete file    n/Esc = cancel",
                    app.theme.key,
                )]),
            ]
        }
        Some(PendingDelete::ScanDirectory { directory }) => vec![
            Line::from(vec![Span::styled("Confirm Directory Delete", app.theme.danger_bold)]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Action: ", app.theme.dim),
                Span::styled("Delete this directory from disk.", app.theme.error),
            ]),
            Line::from(vec![Span::styled("Path:   ", app.theme.dim), Span::raw(directory.as_str())]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "This also removes matching Scan Results and planned moves from the current session.",
                app.theme.warn,
            )]),
            Line::from(""),
            Line::from(vec![Span::styled("y = delete    n/Esc = cancel", app.theme.key)]),
        ],
        None => Vec::new(),
    };

    let dialog = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Double))
        .wrap(Wrap { trim: true });

    frame.render_widget(dialog, dialog_area);
}

fn render_confirmation_dialog(frame: &mut Frame, app: &AppState) {
    let area = frame.area();
    let dialog_area = centered_rect(72, 50, area);

    let mode_msg = if app.dry_run {
        "This is a DRY-RUN. No files will be moved."
    } else {
        "LIVE MODE: pressing y will move files now."
    };

    let mode_style = if app.dry_run { app.theme.warn } else { app.theme.error };

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("⚠  Confirm Move Operation  ⚠", app.theme.danger_bold)]),
        Line::from(""),
        Line::from(vec![Span::styled(mode_msg, mode_style)]),
        Line::from(vec![Span::raw("Reason: dry-run is off; live moves need confirmation.")]),
        Line::from(""),
        Line::from(vec![Span::raw(format!("Source: {}", app.source_path))]),
        Line::from(vec![Span::raw(format!("Items to move: {}", app.identified_items.len()))]),
        Line::from(vec![Span::raw(format!("Conflict policy: {}", app.get_conflict_policy_name()))]),
        Line::from(""),
        Line::from(vec![Span::styled("y = move now    n/Esc = cancel", app.theme.key)]),
    ];

    let confirm = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Double))
        .centered()
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, frame.area());
    frame.render_widget(confirm, dialog_area);
}

fn render_manual_identify_dialog(frame: &mut Frame, app: &AppState) {
    let Some(edit) = &app.manual_edit else {
        return;
    };

    let area = centered_rect(70, 50, frame.area());
    let kind_style = if edit.field == ManualField::Kind { app.theme.strong } else { app.theme.dim };
    let title_style =
        if edit.field == ManualField::Title { app.theme.strong } else { app.theme.dim };
    let year_style = if edit.field == ManualField::Year { app.theme.strong } else { app.theme.dim };
    let kind = match edit.kind {
        rosey_core::MediaKind::Movie => "Movie",
        rosey_core::MediaKind::Episode => "Episode",
        rosey_core::MediaKind::Show => "Show",
        rosey_core::MediaKind::Unknown => "Unknown",
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("Manual Identification", app.theme.header)]),
        Line::from(vec![
            Span::styled("Workflow: ", app.theme.dim),
            Span::raw("edit criteria, search providers, choose result, apply"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Kind:  ", kind_style),
            Span::raw(kind),
            Span::styled(" [m]", app.theme.key),
            Span::raw(" movie "),
            Span::styled("[e]", app.theme.key),
            Span::raw(" episode "),
            Span::styled("[s]", app.theme.key),
            Span::raw(" show "),
            Span::styled("[u]", app.theme.key),
            Span::raw(" unknown"),
        ]),
        Line::from(vec![
            Span::styled("Title: ", title_style),
            Span::raw(if edit.title.is_empty() { "(empty)" } else { edit.title.as_str() }),
        ]),
        Line::from(vec![
            Span::styled("Year:  ", year_style),
            Span::raw(if edit.year.is_empty() { "(none)" } else { edit.year.as_str() }),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Online: ", app.theme.dim),
            Span::raw(edit.search_status.as_str()),
        ]),
    ];

    if !edit.provider_results.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled("Provider Results", app.theme.header)]));
        for (index, result) in edit.provider_results.iter().take(6).enumerate() {
            let marker = if index == edit.provider_index { ">" } else { " " };
            let style = if index == edit.provider_index { app.theme.strong } else { app.theme.dim };
            let year = result.year.map(|year| year.to_string()).unwrap_or_else(|| "N/A".into());
            lines.push(Line::from(vec![Span::styled(
                format!("{marker} {} ({year}) [{}-{}]", result.title, result.provider, result.id),
                style,
            )]));
        }
    }

    lines.extend([
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab", app.theme.key),
            Span::raw(" edit field   "),
            Span::styled("F5", app.theme.key),
            Span::raw(" search providers   "),
            Span::styled("↑/↓", app.theme.key),
            Span::raw(" result   "),
        ]),
        Line::from(vec![
            Span::styled("Enter", app.theme.key),
            Span::raw(" apply selected   "),
            Span::styled("Esc", app.theme.key),
            Span::raw(" cancel"),
        ]),
    ]);

    let dialog = Paragraph::new(lines)
        .block(
            Block::default()
                .title(" Identify ")
                .borders(Borders::ALL)
                .border_type(BorderType::Double),
        )
        .wrap(Wrap { trim: true });

    frame.render_widget(Clear, frame.area());
    frame.render_widget(dialog, area);
}

struct Clear;
impl ratatui::widgets::Widget for Clear {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                buf[(x, y)].set_char(' ').set_style(Style::default());
            }
        }
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

fn render_status_bar(frame: &mut Frame, app: &AppState, area: Rect) {
    let left = format!(
        " {} | {} | {} | {} | {}",
        app.current_screen.title(),
        status_metric(app),
        if app.dry_run { "DRY-RUN" } else { "LIVE" },
        app.get_conflict_policy_name(),
        app.operation_status
    );

    let right =
        if app.is_busy() { " wait  ?:help " } else { " q:quit  s:scan  p:plan  m:move  ?:help " };
    let right_len = right.chars().count();
    let left_budget = area.width as usize;
    let left_budget = left_budget.saturating_sub(right_len);
    let left = truncate_to_width(&left, left_budget);
    let left_len = left.chars().count();

    let status = Line::from(vec![
        Span::styled(left, app.theme.status_left),
        Span::styled(
            " ".repeat((area.width as usize).saturating_sub(left_len + right.len())),
            app.theme.status_fill,
        ),
        Span::styled(right, app.theme.status_right),
    ]);

    let bar = Paragraph::new(status);
    frame.render_widget(bar, area);
}

fn status_metric(app: &AppState) -> String {
    match app.current_screen {
        Screen::ScanResults => {
            let scanned = app.scan_results.len();
            let videos = app.scan_results.iter().filter(|result| result.is_video).count();
            let errors = app.scan_results.iter().filter(|result| result.error.is_some()).count();
            let video_label = count_label(videos, "video", "videos");
            if errors > 0 {
                format!(
                    "{scanned} scanned, {video_label}, {}",
                    count_label(errors, "error", "errors")
                )
            } else {
                format!("{scanned} scanned, {video_label}")
            }
        }
        Screen::PlanPreview => {
            let planned = app.identified_items.len();
            let duplicates =
                app.identified_items.iter().filter(|item| item.is_likely_duplicate()).count();
            if duplicates > 0 {
                format!(
                    "{} planned, {}",
                    planned,
                    count_label(duplicates, "possible duplicate", "possible duplicates")
                )
            } else {
                format!("{} planned", planned)
            }
        }
        Screen::TransferQueue => {
            format!("{}/{} processed", app.transfer_progress.0, app.transfer_queue.len())
        }
        Screen::LogsRecovery => format!("{} logs", app.log_messages.len()),
        Screen::Settings => format!("{} settings", SettingsField::all().len()),
        Screen::Doctor => {
            format!(
                "{} errors, {} warnings",
                app.doctor_report.errors(),
                app.doctor_report.warnings()
            )
        }
        Screen::Dashboard | Screen::Help => format!("{} planned", app.identified_items.len()),
    }
}

fn count_label(count: usize, singular: &str, plural: &str) -> String {
    if count == 1 {
        format!("{count} {singular}")
    } else {
        format!("{count} {plural}")
    }
}

fn active_progress_ratio(app: &AppState) -> Option<f64> {
    if app.transfer_running || app.transfer_progress.1 > 0 {
        return Some(progress_ratio(app.transfer_progress));
    }

    if app.plan_running || app.plan_progress.1 > 0 {
        return Some(progress_ratio(app.plan_progress));
    }

    if (app.scan_running || app.scan_complete) && app.scan_progress.1 > 0 {
        return Some(progress_ratio(app.scan_progress));
    }

    None
}

fn progress_ratio((done, total): (usize, usize)) -> f64 {
    if total == 0 {
        0.0
    } else {
        (done as f64 / total as f64).clamp(0.0, 1.0)
    }
}

fn spinner(tick: u64) -> &'static str {
    const FRAMES: [&str; 4] = ["-", "\\", "|", "/"];
    FRAMES[((tick / 2) as usize) % FRAMES.len()]
}

fn configured_confidence_band(app: &AppState, confidence: u8) -> ConfidenceBand {
    let high = app.confidence_thresholds.green.max(app.confidence_thresholds.yellow);
    let low = app.confidence_thresholds.green.min(app.confidence_thresholds.yellow);

    if confidence >= high {
        ConfidenceBand::Green
    } else if confidence >= low {
        ConfidenceBand::Yellow
    } else {
        ConfidenceBand::Red
    }
}

fn truncate_to_width(value: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let count = value.chars().count();
    if count <= max_width {
        return value.to_string();
    }

    if max_width <= 3 {
        return ".".repeat(max_width);
    }

    let mut truncated: String = value.chars().take(max_width - 3).collect();
    truncated.push_str("...");
    truncated
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{IdentifiedItem, SettingsField};
    use camino::Utf8PathBuf;
    use ratatui::{backend::TestBackend, Terminal};
    use rosey_core::{MediaItem, Score};
    use rosey_fs::ScanResult;

    fn render_text(app: &AppState) -> String {
        let backend = TestBackend::new(100, 32);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| render(frame, app)).unwrap();

        let buffer = terminal.backend().buffer();
        let mut text = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                text.push_str(buffer[(x, y)].symbol());
            }
            text.push('\n');
        }
        text
    }

    fn test_app() -> AppState {
        AppState::new(&rosey_core::RoseyConfig::default(), Utf8PathBuf::from("/source"), None, None)
    }

    #[test]
    fn dashboard_render_smoke() {
        let app = test_app();

        let text = render_text(&app);

        assert!(text.contains("Rosey"));
        assert!(text.contains("Dashboard"));
    }

    #[test]
    fn settings_edit_overlay_render_smoke() {
        let mut app = test_app();
        app.current_screen = Screen::Settings;
        app.selected_settings_index =
            SettingsField::all().iter().position(|field| *field == SettingsField::Source).unwrap();
        app.begin_settings_edit();

        let text = render_text(&app);

        assert!(text.contains("Edit Source"));
        assert!(text.contains("Value:"));
    }

    #[test]
    fn scan_results_explain_identify_action() {
        let mut app = test_app();
        app.current_screen = Screen::ScanResults;
        app.scan_results.push(ScanResult {
            path: Utf8PathBuf::from("/source/Movie.mkv"),
            is_video: true,
            size_bytes: 1_000,
            error: None,
        });

        let text = render_text(&app);

        assert!(text.contains("Identify Source"));
        assert!(text.contains("press i to identify"));
        assert!(text.contains("i=identify"));
    }

    #[test]
    fn scan_status_bar_reports_scan_counts() {
        let mut app = test_app();
        app.current_screen = Screen::ScanResults;
        app.scan_results.push(ScanResult {
            path: Utf8PathBuf::from("/source/Movie.mkv"),
            is_video: true,
            size_bytes: 1_000,
            error: None,
        });
        app.scan_results.push(ScanResult {
            path: Utf8PathBuf::from("/source/Broken.mkv"),
            is_video: false,
            size_bytes: 0,
            error: Some("denied".into()),
        });

        let text = render_text(&app);

        assert!(text.contains("Scan Results | 2 scanned, 1 video, 1 error"));
    }

    #[test]
    fn live_move_confirmation_explains_why_and_next_keys() {
        let mut app = test_app();
        app.dry_run = false;
        app.show_confirmation = true;
        app.identified_items.push(IdentifiedItem {
            media_item: MediaItem {
                kind: MediaKind::Movie,
                source_path: Utf8PathBuf::from("/source/Movie.mkv"),
                title: Some("Movie".into()),
                year: Some(2020),
                season: None,
                episodes: Vec::new(),
                part: None,
                date: None,
                sidecars: Vec::new(),
                nfo: Default::default(),
            },
            score: Score { confidence: 80, reasons: Vec::new() },
            destination: Utf8PathBuf::from("/movies/Movie (2020)/Movie.mkv"),
        });

        let text = render_text(&app);

        assert!(text.contains("LIVE MODE"));
        assert!(text.contains("dry-run is off"));
        assert!(text.contains("Items to move: 1"));
        assert!(text.contains("y = move now"));
        assert!(text.contains("n/Esc = cancel"));
    }

    #[test]
    fn delete_confirmation_explains_plan_removal() {
        let mut app = test_app();
        app.pending_delete = Some(PendingDelete::PlanItem { item_index: 0 });
        app.identified_items.push(IdentifiedItem {
            media_item: MediaItem {
                kind: MediaKind::Movie,
                source_path: Utf8PathBuf::from("/source/Movie.mkv"),
                title: Some("Movie".into()),
                year: Some(2020),
                season: None,
                episodes: Vec::new(),
                part: None,
                date: None,
                sidecars: Vec::new(),
                nfo: Default::default(),
            },
            score: Score { confidence: 80, reasons: Vec::new() },
            destination: Utf8PathBuf::from("/movies/Movie (2020)/Movie.mkv"),
        });

        let text = render_text(&app);

        assert!(text.contains("Confirm Plan Removal"));
        assert!(text.contains("y removes only from the plan"));
        assert!(text.contains("Del deletes the source file"));
        assert!(text.contains("y = remove"));
        assert!(text.contains("Del = delete file"));
    }

    #[test]
    fn delete_confirmation_explains_scan_directory_delete() {
        let mut app = test_app();
        app.pending_delete = Some(PendingDelete::ScanDirectory {
            directory: Utf8PathBuf::from("/source/Movie (2020)"),
        });

        let text = render_text(&app);

        assert!(text.contains("Confirm Directory Delete"));
        assert!(text.contains("Delete this directory from disk"));
        assert!(text.contains("y = delete"));
    }

    #[test]
    fn doctor_render_smoke() {
        let mut app = test_app();
        app.current_screen = Screen::Doctor;

        let text = render_text(&app);

        assert!(text.contains("Doctor"));
        assert!(text.contains("Overall"));
    }

    #[test]
    fn plan_preview_prompts_to_build_plan_after_scan() {
        let mut app = test_app();
        app.current_screen = Screen::PlanPreview;
        app.scan_results.push(ScanResult {
            path: Utf8PathBuf::from("/source/Movie.mkv"),
            is_video: true,
            size_bytes: 1_000,
            error: None,
        });

        let text = render_text(&app);

        assert!(text.contains("Source -> Destination"));
        assert!(text.contains("Press p to build"));
    }

    #[test]
    fn plan_preview_shows_selected_source_and_destination() {
        let mut app = test_app();
        app.current_screen = Screen::PlanPreview;
        app.identified_items.push(IdentifiedItem {
            media_item: MediaItem {
                kind: MediaKind::Movie,
                source_path: Utf8PathBuf::from("/source/Movie.mkv"),
                title: Some("Movie".into()),
                year: Some(2020),
                season: None,
                episodes: Vec::new(),
                part: None,
                date: None,
                sidecars: Vec::new(),
                nfo: Default::default(),
            },
            score: Score { confidence: 90, reasons: Vec::new() },
            destination: Utf8PathBuf::from("/movies/Movie (2020)/Movie.mkv"),
        });
        app.rebuild_filtered();

        let text = render_text(&app);

        assert!(text.contains("From:"));
        assert!(text.contains("To:"));
        assert!(text.contains("Source"));
        assert!(text.contains("Destination"));
        assert!(text.contains("/source/Movie.mkv"));
    }

    #[test]
    fn plan_preview_scrolls_table_to_selected_item() {
        let mut app = test_app();
        app.current_screen = Screen::PlanPreview;
        for index in 0..25 {
            let title = format!("Movie {index:02}");
            app.identified_items.push(IdentifiedItem {
                media_item: MediaItem {
                    kind: MediaKind::Movie,
                    source_path: Utf8PathBuf::from(format!("/source/{title}.mkv")),
                    title: Some(title.clone()),
                    year: Some(2020),
                    season: None,
                    episodes: Vec::new(),
                    part: None,
                    date: None,
                    sidecars: Vec::new(),
                    nfo: Default::default(),
                },
                score: Score { confidence: 80, reasons: Vec::new() },
                destination: Utf8PathBuf::from(format!("/movies/{title} (2020)/{title}.mkv")),
            });
        }
        app.rebuild_filtered();
        app.selected_index = 20;

        let text = render_text(&app);

        assert!(text.contains("Movie 20"));
        assert!(text.contains("rows"));
        assert!(!text.contains("Movie 00"), "{text}");
    }

    #[test]
    fn plan_visible_window_keeps_selection_visible() {
        assert_eq!(plan_visible_window(0, 25, 14), (0, 11));
        assert_eq!(plan_visible_window(20, 25, 14), (10, 21));
        assert_eq!(plan_visible_window(24, 25, 14), (14, 25));
    }

    #[test]
    fn plan_preview_flags_existing_destination_directory_as_duplicate() {
        let temp_root = std::env::temp_dir()
            .join(format!("rosey-duplicate-render-test-{}", std::process::id()));
        let duplicate_dir = temp_root.join("Movie (2020)");
        std::fs::create_dir_all(&duplicate_dir).unwrap();
        let duplicate_dir = Utf8PathBuf::from_path_buf(duplicate_dir).unwrap();

        let mut app = test_app();
        app.current_screen = Screen::PlanPreview;
        app.identified_items.push(IdentifiedItem {
            media_item: MediaItem {
                kind: MediaKind::Movie,
                source_path: Utf8PathBuf::from("/source/Movie.mkv"),
                title: Some("Movie".into()),
                year: Some(2020),
                season: None,
                episodes: Vec::new(),
                part: None,
                date: None,
                sidecars: Vec::new(),
                nfo: Default::default(),
            },
            score: Score { confidence: 90, reasons: Vec::new() },
            destination: duplicate_dir.join("Movie (2020).mkv"),
        });
        app.rebuild_filtered();

        let text = render_text(&app);

        assert!(text.contains("Dup"));
        assert!(text.contains("DIR"), "{text}");
        assert!(text.contains("possible duplicate"));
        assert!(text.contains("destination directory already exists"));

        let _ = std::fs::remove_dir_all(temp_root);
    }

    #[test]
    fn identify_overlay_render_smoke() {
        let mut app = test_app();
        app.current_screen = Screen::PlanPreview;
        app.identified_items.push(IdentifiedItem {
            media_item: MediaItem {
                kind: MediaKind::Movie,
                source_path: Utf8PathBuf::from("/source/Movie.mkv"),
                title: Some("Movie".into()),
                year: Some(2020),
                season: None,
                episodes: Vec::new(),
                part: None,
                date: None,
                sidecars: Vec::new(),
                nfo: Default::default(),
            },
            score: Score { confidence: 50, reasons: Vec::new() },
            destination: Utf8PathBuf::from("/movies/Movie (2020)/Movie.mkv"),
        });
        app.rebuild_filtered();
        app.begin_manual_edit();

        let text = render_text(&app);

        assert!(text.contains("Manual Identification"));
        assert!(text.contains("Workflow:"));
        assert!(text.contains("edit criteria, search providers"));
        assert!(text.contains("apply"));
    }

    #[test]
    fn help_identify_instructions_are_clear() {
        let mut app = test_app();
        app.current_screen = Screen::Help;
        let text = render_text(&app);

        assert!(text.contains("Identify selected scan result or planned move"));
        assert!(text.contains("Search providers from manual identify"));
    }
}
