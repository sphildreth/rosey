use crate::app::{
    AppState, ManualField, Screen, SettingsField, SortColumn, SortDirection, TransferState,
};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Table, Wrap,
    },
    Frame,
};
use rosey_core::{ConfidenceBand, DoctorStatus, MediaKind};

const HEADER_STYLE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
const GREEN_STYLE: Style = Style::new().fg(Color::Green);
const YELLOW_STYLE: Style = Style::new().fg(Color::Yellow);
const RED_STYLE: Style = Style::new().fg(Color::Red);
const GRAY_STYLE: Style = Style::new().fg(Color::Gray);
const WHITE_BOLD: Style = Style::new().fg(Color::White).add_modifier(Modifier::BOLD);
const HIGHLIGHT_STYLE: Style = Style::new().bg(Color::DarkGray).add_modifier(Modifier::BOLD);

pub fn render(frame: &mut Frame, app: &AppState) {
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

fn render_header(frame: &mut Frame, _app: &AppState, area: Rect) {
    let title = Paragraph::new(Text::from(vec![
        Line::from(vec![Span::styled("Rosey", Style::new().fg(Color::Magenta).bold())]),
        Line::from(vec![Span::raw("Media File Organizer")]),
    ]))
    .block(Block::default().borders(Borders::ALL).border_style(Style::new().fg(Color::Magenta)))
    .centered();

    frame.render_widget(title, area);
}

fn render_tabs(frame: &mut Frame, app: &AppState, area: Rect) {
    let tabs: Vec<Span> = Screen::all()
        .iter()
        .map(|s| {
            let label = format!(" {}:{} ", s.shortcut(), s.title());
            if *s == app.current_screen {
                Span::styled(label, Style::new().fg(Color::Black).bg(Color::Cyan).bold())
            } else {
                Span::styled(label, GRAY_STYLE)
            }
        })
        .collect();

    let tab_line = Line::from(tabs);
    let tab_widget = Paragraph::new(tab_line).block(Block::default());
    frame.render_widget(tab_widget, area);
}

fn render_dashboard(frame: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    let info_lines = vec![
        Line::from(vec![
            Span::styled("Source:         ", GRAY_STYLE),
            Span::styled(format!("{}", app.source_path), WHITE_BOLD),
        ]),
        Line::from(vec![
            Span::styled("Movies Target:  ", GRAY_STYLE),
            Span::styled(
                app.movies_target.as_ref().map(|p| p.as_str()).unwrap_or("(not set)"),
                WHITE_BOLD,
            ),
        ]),
        Line::from(vec![
            Span::styled("TV Target:      ", GRAY_STYLE),
            Span::styled(
                app.tv_target.as_ref().map(|p| p.as_str()).unwrap_or("(not set)"),
                WHITE_BOLD,
            ),
        ]),
        Line::from(vec![
            Span::styled("Mode:           ", GRAY_STYLE),
            Span::styled(
                if app.dry_run { "DRY-RUN" } else { "LIVE" },
                if app.dry_run { YELLOW_STYLE } else { RED_STYLE },
            ),
        ]),
        Line::from(vec![
            Span::styled("Conflict:       ", GRAY_STYLE),
            Span::styled(app.get_conflict_policy_name(), WHITE_BOLD),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("s", Style::new().fg(Color::Cyan).bold()),
            Span::raw(" scan    "),
            Span::styled("p", Style::new().fg(Color::Cyan).bold()),
            Span::raw(" plan    "),
            Span::styled("m", Style::new().fg(Color::Cyan).bold()),
            Span::raw(" move    "),
            Span::styled("?", Style::new().fg(Color::Cyan).bold()),
            Span::raw(" help    "),
            Span::styled("q", Style::new().fg(Color::Cyan).bold()),
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
            Span::styled("Scanned:  ", GRAY_STYLE),
            Span::raw(format!(
                "{} video files",
                app.scan_results.iter().filter(|r| r.is_video).count()
            )),
        ]),
        Line::from(vec![
            Span::styled("Planned:  ", GRAY_STYLE),
            Span::raw(format!("{} items", app.identified_items.len())),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Green:    ", GREEN_STYLE),
            Span::raw(format!(
                "{} ",
                app.identified_items.iter().filter(|i| i.score.confidence >= high).count()
            )),
            Span::styled("Yellow:   ", YELLOW_STYLE),
            Span::raw(format!(
                "{} ",
                app.identified_items
                    .iter()
                    .filter(|i| (low..high).contains(&i.score.confidence))
                    .count()
            )),
            Span::styled("Red:      ", RED_STYLE),
            Span::raw(format!(
                "{}",
                app.identified_items.iter().filter(|i| i.score.confidence < low).count()
            )),
        ]),
        Line::from(vec![
            Span::styled("Transferred: ", GRAY_STYLE),
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
        RED_STYLE
    } else if app.is_busy() {
        Style::new().fg(Color::Cyan)
    } else {
        GREEN_STYLE
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
                .style(RED_STYLE)
                .block(Block::default().borders(Borders::ALL).border_style(Style::new().red()));

        frame.render_widget(error, area);
        return;
    }

    let table_area = if app.scan_running {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);
        render_activity(frame, app, chunks[0]);
        chunks[1]
    } else {
        area
    };

    let rows: Vec<Row> = app
        .scan_results
        .iter()
        .map(|r| {
            Row::new(vec![
                Cell::from(r.path.as_str()),
                Cell::from(if r.is_video { "video" } else { "other" }),
                Cell::from(format!("{}", r.size_bytes)),
                Cell::from(r.error.as_deref().unwrap_or("")),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(50),
        Constraint::Percentage(10),
        Constraint::Percentage(15),
        Constraint::Percentage(25),
    ];

    let table = Table::new(rows, widths)
        .header(Row::new(vec!["Path", "Type", "Size", "Error"]).style(HEADER_STYLE))
        .block(Block::default().title(" Scan Results ").borders(Borders::ALL));

    frame.render_widget(table, table_area);
}

fn render_plan_preview(frame: &mut Frame, app: &AppState, area: Rect) {
    let constraints = if app.plan_running {
        vec![Constraint::Length(3), Constraint::Length(3), Constraint::Min(1)]
    } else {
        vec![Constraint::Length(3), Constraint::Min(1)]
    };

    let chunks =
        Layout::default().direction(Direction::Vertical).constraints(constraints).split(area);
    let table_area = if app.plan_running {
        render_activity(frame, app, chunks[1]);
        chunks[2]
    } else {
        chunks[1]
    };

    let search = Paragraph::new(if app.filter_input_active {
        format!("Filter: {}▌", app.search_query)
    } else {
        format!("Filter: {} (press / to search)", app.search_query)
    })
    .block(Block::default().borders(Borders::ALL));

    frame.render_widget(search, chunks[0]);

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

    let rows: Vec<Row> = app
        .filtered_items
        .iter()
        .enumerate()
        .map(|(idx, &item_idx)| {
            let item = &app.identified_items[item_idx];
            let style = match configured_confidence_band(app, item.score.confidence) {
                ConfidenceBand::Green => GREEN_STYLE,
                ConfidenceBand::Yellow => YELLOW_STYLE,
                ConfidenceBand::Red => RED_STYLE,
            };

            let row_style = if idx == app.selected_index { HIGHLIGHT_STYLE } else { Style::new() };

            let kind_str = match item.media_item.kind {
                MediaKind::Movie => "Movie",
                MediaKind::Episode => "Episode",
                MediaKind::Show => "Show",
                MediaKind::Unknown => "?",
            };

            Row::new(vec![
                Cell::from(format!("{:3}%", item.score.confidence)).style(style),
                Cell::from(kind_str),
                Cell::from(item.media_item.title.as_deref().unwrap_or("?")),
                Cell::from(
                    item.media_item.year.map(|y| y.to_string()).unwrap_or_else(|| "-".to_string()),
                ),
                Cell::from(item.destination.as_str()),
            ])
            .style(row_style)
        })
        .collect();

    let widths = [
        Constraint::Length(5),
        Constraint::Length(7),
        Constraint::Percentage(30),
        Constraint::Length(6),
        Constraint::Percentage(50),
    ];

    let table = Table::new(rows, widths)
        .header(Row::new(vec!["Conf", "Kind", "Title", "Year", "Destination"]).style(HEADER_STYLE))
        .block(
            Block::default()
                .title(format!(" Plan Preview ({}) - {} ", app.filtered_items.len(), sort_hint))
                .borders(Borders::ALL),
        );

    frame.render_widget(table, table_area);
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
        .gauge_style(Style::new().fg(Color::Cyan))
        .label(format!("Progress: {}/{}", app.transfer_progress.0, app.transfer_progress.1))
        .ratio(progress);

    frame.render_widget(gauge, chunks[0]);

    let rows: Vec<Row> = app
        .transfer_queue
        .iter()
        .map(|t| {
            let (state_str, state_style) = match t.state {
                TransferState::Pending => ("PENDING", GRAY_STYLE),
                TransferState::InProgress => ("MOVING...", YELLOW_STYLE),
                TransferState::WouldMove => ("WOULD MOVE", GREEN_STYLE),
                TransferState::Completed => ("DONE", GREEN_STYLE),
                TransferState::Failed => ("FAILED", RED_STYLE),
                TransferState::Skipped => ("SKIPPED", YELLOW_STYLE),
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
        .header(Row::new(vec!["Status", "Title", "Destination", "Error"]).style(HEADER_STYLE))
        .block(
            Block::default()
                .title(format!(" Transfer Queue ({}) ", app.transfer_queue.len()))
                .borders(Borders::ALL),
        );

    frame.render_widget(table, chunks[1]);
}

fn render_logs(frame: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(1)])
        .split(area);

    let recovery = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("Journal:  ", GRAY_STYLE),
            Span::raw(app.last_journal_path.as_ref().map(|p| p.as_str()).unwrap_or("(none)")),
        ]),
        Line::from(vec![
            Span::styled("Recovery: ", GRAY_STYLE),
            Span::raw(app.recovery_summary.as_str()),
        ]),
        Line::from(vec![
            Span::styled("r", Style::new().fg(Color::Cyan).bold()),
            Span::raw(" inspect last move journal"),
        ]),
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
        Span::styled("Up/Down ", Style::new().fg(Color::Cyan).bold()),
        Span::raw("select   "),
        Span::styled("e ", Style::new().fg(Color::Cyan).bold()),
        Span::raw("edit   "),
        Span::styled("w/s ", Style::new().fg(Color::Cyan).bold()),
        Span::raw("save"),
    ]));
    settings_lines.push(Line::from(""));

    for (index, field) in SettingsField::all().iter().enumerate() {
        let selected = index == app.selected_settings_index;
        let marker = if selected { ">" } else { " " };
        let label_style = if selected { WHITE_BOLD } else { GRAY_STYLE };
        let value = app.settings_value(*field);
        settings_lines.push(Line::from(vec![
            Span::styled(format!("{marker} {:<22}", field.label()), label_style),
            Span::raw(truncate_to_width(&value, area.width.saturating_sub(26) as usize)),
        ]));
    }

    settings_lines.push(Line::from(""));
    settings_lines.push(Line::from(vec![
        Span::styled("Status: ", GRAY_STYLE),
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
        Line::from(vec![Span::styled(format!("Edit {}", edit.field.label()), HEADER_STYLE)]),
        Line::from(""),
        Line::from(vec![Span::styled("Value: ", GRAY_STYLE), Span::raw(edit.value.as_str())]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Enter", Style::new().fg(Color::Cyan).bold()),
            Span::raw(" apply   "),
            Span::styled("Esc", Style::new().fg(Color::Cyan).bold()),
            Span::raw(" cancel   "),
            Span::styled("Backspace", Style::new().fg(Color::Cyan).bold()),
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
    let mut lines = vec![
        Line::from(vec![
            Span::styled("Overall: ", GRAY_STYLE),
            Span::styled(doctor_status_label(report.overall), doctor_status_style(report.overall)),
            Span::raw(format!("  {} errors, {} warnings", report.errors(), report.warnings())),
        ]),
        Line::from(vec![Span::styled("Config:  ", GRAY_STYLE), Span::raw(&report.config_path)]),
        Line::from(vec![
            Span::styled("Keys:    ", GRAY_STYLE),
            Span::raw("o refresh, Up/Down scroll, PgUp/PgDn page"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled("Attention", HEADER_STYLE)]),
    ];

    let issues: Vec<_> =
        report.checks.iter().filter(|check| check.status != DoctorStatus::Ok).collect();
    if issues.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("[OK] ", GREEN_STYLE),
            Span::raw("No warnings or errors."),
        ]));
    } else {
        for check in &issues {
            lines.push(doctor_check_line(check));
            if let Some(detail) = &check.detail {
                lines.push(Line::from(vec![Span::raw("      "), Span::styled(detail, GRAY_STYLE)]));
            }
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled("All Checks", HEADER_STYLE)]));
    for check in &report.checks {
        lines.push(doctor_check_line(check));
        if let Some(detail) = &check.detail {
            lines.push(Line::from(vec![Span::raw("      "), Span::styled(detail, GRAY_STYLE)]));
        }
    }

    let scroll = app.doctor_scroll.min(lines.len().saturating_sub(1)) as u16;
    let doctor = Paragraph::new(lines)
        .block(Block::default().title(" Doctor ").borders(Borders::ALL))
        .scroll((scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(doctor, area);
}

fn doctor_check_line(check: &rosey_core::DoctorCheck) -> Line<'_> {
    Line::from(vec![
        Span::styled(
            format!("[{}] ", doctor_status_label(check.status)),
            doctor_status_style(check.status),
        ),
        Span::styled(format!("{}: ", check.name), WHITE_BOLD),
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

fn doctor_status_style(status: DoctorStatus) -> Style {
    match status {
        DoctorStatus::Ok => GREEN_STYLE,
        DoctorStatus::Warn => YELLOW_STYLE,
        DoctorStatus::Error => RED_STYLE,
    }
}

fn render_help(frame: &mut Frame, _app: &AppState, area: Rect) {
    let help_lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("Keyboard Shortcuts", Style::new().fg(Color::Cyan).bold())]),
        Line::from(""),
        Line::from(vec![
            Span::styled("1-8 ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Switch screens"),
        ]),
        Line::from(vec![
            Span::styled("s  ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Scan source directory"),
        ]),
        Line::from(vec![
            Span::styled("p  ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Plan (identify + score all scanned files)"),
        ]),
        Line::from(vec![
            Span::styled("m  ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Execute move (requires confirmation in live mode)"),
        ]),
        Line::from(vec![
            Span::styled("d  ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Toggle dry-run mode"),
        ]),
        Line::from(vec![
            Span::styled("c  ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Cycle conflict policy (Skip → Replace → Keep Both)"),
        ]),
        Line::from(vec![
            Span::styled("/  ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Search/filter in Plan Preview"),
        ]),
        Line::from(vec![
            Span::styled("i  ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Manually identify selected plan item"),
        ]),
        Line::from(vec![
            Span::styled("F5 ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Search online providers from identify overlay"),
        ]),
        Line::from(vec![
            Span::styled("r  ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Inspect last move journal on Logs / Recovery"),
        ]),
        Line::from(vec![
            Span::styled("x  ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Clean moved source folders on Transfer Queue"),
        ]),
        Line::from(vec![
            Span::styled("o  ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Refresh Doctor checks"),
        ]),
        Line::from(vec![
            Span::styled("Esc", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Clear filter"),
        ]),
        Line::from(vec![
            Span::styled("↑↓ ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Navigate items"),
        ]),
        Line::from(vec![
            Span::styled("+/-", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Adjust confidence threshold"),
        ]),
        Line::from(vec![
            Span::styled("e  ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Edit selected Settings field"),
        ]),
        Line::from(vec![
            Span::styled("y/n", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Confirm/cancel in confirmation dialog"),
        ]),
        Line::from(vec![
            Span::styled("q  ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Quit"),
        ]),
        Line::from(vec![
            Span::styled("?  ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("Toggle help"),
        ]),
    ];

    let help =
        Paragraph::new(help_lines).block(Block::default().title(" Help ").borders(Borders::ALL));

    frame.render_widget(help, area);
}

fn render_confirmation_dialog(frame: &mut Frame, app: &AppState) {
    let area = frame.area();
    let dialog_area = centered_rect(60, 20, area);

    let mode_msg = if app.dry_run {
        "This is a DRY-RUN. No files will be moved."
    } else {
        "LIVE MODE: Files WILL be moved/destroyed."
    };

    let mode_style = if app.dry_run { YELLOW_STYLE } else { RED_STYLE };

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "⚠  Confirm Move Operation  ⚠",
            Style::new().fg(Color::Yellow).bold(),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(mode_msg, mode_style)]),
        Line::from(""),
        Line::from(vec![Span::raw(format!("Source: {}", app.source_path))]),
        Line::from(vec![Span::raw(format!("Items to move: {}", app.identified_items.len()))]),
        Line::from(vec![Span::raw(format!("Conflict policy: {}", app.get_conflict_policy_name()))]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Press [y] to confirm or [n] to cancel",
            Style::new().fg(Color::Cyan).bold(),
        )]),
    ];

    let confirm = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Double))
        .centered();

    frame.render_widget(Clear, frame.area());
    frame.render_widget(confirm, dialog_area);
}

fn render_manual_identify_dialog(frame: &mut Frame, app: &AppState) {
    let Some(edit) = &app.manual_edit else {
        return;
    };

    let area = centered_rect(70, 50, frame.area());
    let kind_style = if edit.field == ManualField::Kind { WHITE_BOLD } else { GRAY_STYLE };
    let title_style = if edit.field == ManualField::Title { WHITE_BOLD } else { GRAY_STYLE };
    let year_style = if edit.field == ManualField::Year { WHITE_BOLD } else { GRAY_STYLE };
    let kind = match edit.kind {
        rosey_core::MediaKind::Movie => "Movie",
        rosey_core::MediaKind::Episode => "Episode",
        rosey_core::MediaKind::Show => "Show",
        rosey_core::MediaKind::Unknown => "Unknown",
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("Manual Identification", HEADER_STYLE)]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Kind:  ", kind_style),
            Span::raw(kind),
            Span::styled("   m", Style::new().fg(Color::Cyan).bold()),
            Span::raw(" movie "),
            Span::styled("e", Style::new().fg(Color::Cyan).bold()),
            Span::raw(" episode "),
            Span::styled("u", Style::new().fg(Color::Cyan).bold()),
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
            Span::styled("Online: ", GRAY_STYLE),
            Span::raw(edit.search_status.as_str()),
        ]),
    ];

    if !edit.provider_results.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled("Provider Results", HEADER_STYLE)]));
        for (index, result) in edit.provider_results.iter().take(6).enumerate() {
            let marker = if index == edit.provider_index { ">" } else { " " };
            let style = if index == edit.provider_index { WHITE_BOLD } else { GRAY_STYLE };
            let year = result.year.map(|year| year.to_string()).unwrap_or_else(|| "N/A".into());
            lines.push(Line::from(vec![Span::styled(
                format!("{marker} {} ({year}) [tmdbid-{}]", result.title, result.id),
                style,
            )]));
        }
    }

    lines.extend([
        Line::from(""),
        Line::from(vec![
            Span::styled("Tab", Style::new().fg(Color::Cyan).bold()),
            Span::raw(" field   "),
            Span::styled("F5", Style::new().fg(Color::Cyan).bold()),
            Span::raw(" search   "),
            Span::styled("↑/↓", Style::new().fg(Color::Cyan).bold()),
            Span::raw(" result   "),
        ]),
        Line::from(vec![
            Span::styled("Enter", Style::new().fg(Color::Cyan).bold()),
            Span::raw(" apply   "),
            Span::styled("Esc", Style::new().fg(Color::Cyan).bold()),
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
        " {} | {} items | {} | {} | {}",
        app.current_screen.title(),
        app.identified_items.len(),
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
        Span::styled(left, Style::new().fg(Color::White).bg(Color::Rgb(40, 40, 40))),
        Span::styled(
            " ".repeat((area.width as usize).saturating_sub(left_len + right.len())),
            Style::new().bg(Color::Rgb(40, 40, 40)),
        ),
        Span::styled(right, Style::new().fg(Color::Gray).bg(Color::Rgb(40, 40, 40))),
    ]);

    let bar = Paragraph::new(status);
    frame.render_widget(bar, area);
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
    fn doctor_render_smoke() {
        let mut app = test_app();
        app.current_screen = Screen::Doctor;

        let text = render_text(&app);

        assert!(text.contains("Doctor"));
        assert!(text.contains("Overall"));
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
        assert!(text.contains("F5"));
    }
}
