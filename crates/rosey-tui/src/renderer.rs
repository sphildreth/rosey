use crate::app::{AppState, Screen, SortColumn, SortDirection, TransferState};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Gauge, List, ListItem, Paragraph, Row, Table, Wrap,
    },
    Frame,
};
use rosey_core::{ConfidenceBand, MediaKind};

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
        Screen::Help => render_help(frame, app, chunks[2]),
    }

    render_status_bar(frame, app, chunks[3]);
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
        .constraints([Constraint::Length(8), Constraint::Min(1)])
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

    let stats_lines = vec![
        Line::from(vec![
            Span::styled("Scanned:  ", GRAY_STYLE),
            Span::raw(format!("{} video files", app.scan_results.len())),
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
                app.identified_items.iter().filter(|i| i.score.confidence >= 70).count()
            )),
            Span::styled("Yellow:   ", YELLOW_STYLE),
            Span::raw(format!(
                "{} ",
                app.identified_items
                    .iter()
                    .filter(|i| (40..70).contains(&i.score.confidence))
                    .count()
            )),
            Span::styled("Red:      ", RED_STYLE),
            Span::raw(format!(
                "{}",
                app.identified_items.iter().filter(|i| i.score.confidence < 40).count()
            )),
        ]),
        Line::from(vec![
            Span::styled("Transferred: ", GRAY_STYLE),
            Span::raw(format!("{}/{}", app.transfer_progress.0, app.transfer_queue.len())),
        ]),
    ];

    let stats =
        Paragraph::new(stats_lines).block(Block::default().title(" Status ").borders(Borders::ALL));

    frame.render_widget(stats, chunks[1]);
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

    if app.scan_running {
        let gauge = Gauge::default()
            .gauge_style(Style::new().fg(Color::Cyan))
            .label("Scanning...")
            .block(Block::default().title(" Scanning ").borders(Borders::ALL));

        frame.render_widget(gauge, area);
        return;
    }

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

    frame.render_widget(table, area);
}

fn render_plan_preview(frame: &mut Frame, app: &AppState, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

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
            let style = match rosey_core::confidence_band(item.score.confidence) {
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

    frame.render_widget(table, chunks[1]);
}

fn render_transfer_queue(frame: &mut Frame, app: &AppState, area: Rect) {
    if app.transfer_queue.is_empty() {
        let empty = Paragraph::new(
            "No items in transfer queue.\n\nAdd items from Plan Preview by selecting them and pressing 'a' or 'm'.",
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
        Constraint::Length(8),
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
    let items: Vec<ListItem> =
        app.log_messages.iter().rev().take(50).map(|msg| ListItem::new(Span::raw(msg))).collect();

    let list = List::new(items).block(
        Block::default().title(format!(" Log ({}) ", app.log_messages.len())).borders(Borders::ALL),
    );

    frame.render_widget(list, area);
}

fn render_settings(frame: &mut Frame, app: &AppState, area: Rect) {
    let settings_lines = vec![
        Line::from(vec![
            Span::styled("Source:           ", GRAY_STYLE),
            Span::raw(app.source_path.as_str()),
        ]),
        Line::from(vec![
            Span::styled("Movies Target:    ", GRAY_STYLE),
            Span::raw(app.movies_target.as_ref().map(|p| p.as_str()).unwrap_or("")),
        ]),
        Line::from(vec![
            Span::styled("TV Target:        ", GRAY_STYLE),
            Span::raw(app.tv_target.as_ref().map(|p| p.as_str()).unwrap_or("")),
        ]),
        Line::from(vec![
            Span::styled("Dry-run:          ", GRAY_STYLE),
            Span::raw(format!("{}", app.dry_run)),
        ]),
        Line::from(vec![
            Span::styled("Conflict Policy:  ", GRAY_STYLE),
            Span::raw(app.get_conflict_policy_name()),
        ]),
        Line::from(vec![
            Span::styled("Max Workers:      ", GRAY_STYLE),
            Span::raw(format!("{}", app.max_workers)),
        ]),
        Line::from(vec![
            Span::styled("Confidence Floor: ", GRAY_STYLE),
            Span::raw(format!("{}", app.confidence_threshold)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("c ", Style::new().fg(Color::Cyan).bold()),
            Span::raw("conflict   "),
            Span::styled("d", Style::new().fg(Color::Cyan).bold()),
            Span::raw(" dry-run toggle   "),
            Span::styled("+/-", Style::new().fg(Color::Cyan).bold()),
            Span::raw(" confidence"),
        ]),
    ];

    let settings = Paragraph::new(settings_lines)
        .block(Block::default().title(" Settings ").borders(Borders::ALL))
        .wrap(Wrap { trim: true });

    frame.render_widget(settings, area);
}

fn render_help(frame: &mut Frame, _app: &AppState, area: Rect) {
    let help_lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled("Keyboard Shortcuts", Style::new().fg(Color::Cyan).bold())]),
        Line::from(""),
        Line::from(vec![
            Span::styled("1-7 ", Style::new().fg(Color::Cyan).bold()),
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
        " {} | {} items | {} mode | {}",
        app.current_screen.title(),
        app.identified_items.len(),
        if app.dry_run { "DRY-RUN" } else { "LIVE" },
        app.get_conflict_policy_name()
    );

    let right = " q:quit  s:scan  p:plan  m:move  ?:help ";
    let left_len = left.len();

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
