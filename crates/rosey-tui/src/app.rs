use camino::Utf8PathBuf;
use rosey_core::{ConfidenceThresholds, ConflictPolicy, MediaItem, RoseyConfig, Score};
use rosey_fs::ScanResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Dashboard,
    ScanResults,
    PlanPreview,
    TransferQueue,
    LogsRecovery,
    Settings,
    Help,
}

impl Screen {
    pub fn all() -> &'static [Screen] {
        &[
            Screen::Dashboard,
            Screen::ScanResults,
            Screen::PlanPreview,
            Screen::TransferQueue,
            Screen::LogsRecovery,
            Screen::Settings,
            Screen::Help,
        ]
    }

    pub fn title(&self) -> &'static str {
        match self {
            Screen::Dashboard => "Dashboard",
            Screen::ScanResults => "Scan Results",
            Screen::PlanPreview => "Plan Preview",
            Screen::TransferQueue => "Transfer Queue",
            Screen::LogsRecovery => "Logs / Recovery",
            Screen::Settings => "Settings",
            Screen::Help => "Help",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            Screen::Dashboard => "1",
            Screen::ScanResults => "2",
            Screen::PlanPreview => "3",
            Screen::TransferQueue => "4",
            Screen::LogsRecovery => "5",
            Screen::Settings => "6",
            Screen::Help => "7",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifiedItem {
    pub media_item: MediaItem,
    pub score: Score,
    pub destination: Utf8PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TransferState {
    Pending,
    InProgress,
    WouldMove,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone)]
pub struct TransferItem {
    pub item: IdentifiedItem,
    pub state: TransferState,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SortColumn {
    Title,
    Year,
    Confidence,
    Kind,
    Destination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

pub struct AppState {
    pub current_screen: Screen,
    pub should_quit: bool,
    pub show_confirmation: bool,

    pub source_path: Utf8PathBuf,
    pub movies_target: Option<Utf8PathBuf>,
    pub tv_target: Option<Utf8PathBuf>,
    pub dry_run: bool,
    pub conflict_policy: ConflictPolicy,
    pub max_workers: usize,
    pub follow_symlinks: bool,
    pub confidence_threshold: u8,
    pub confidence_thresholds: ConfidenceThresholds,
    pub operation_status: String,
    pub operation_detail: String,
    pub activity_tick: u64,

    pub scan_results: Vec<ScanResult>,
    pub scan_running: bool,
    pub scan_error: Option<String>,
    pub scan_complete: bool,
    pub scan_progress: (usize, usize),

    pub identified_items: Vec<IdentifiedItem>,
    pub filtered_items: Vec<usize>,
    pub search_query: String,
    pub sort_column: SortColumn,
    pub sort_direction: SortDirection,
    pub plan_running: bool,
    pub plan_progress: (usize, usize),

    pub selected_index: usize,
    pub transfer_queue: Vec<TransferItem>,
    pub transfer_running: bool,
    pub transfer_progress: (usize, usize),
    pub transfer_success_count: usize,

    pub log_messages: Vec<String>,
    pub filter_input_active: bool,
}

impl AppState {
    pub fn new(
        config: &RoseyConfig,
        source_path: Utf8PathBuf,
        movies_target: Option<Utf8PathBuf>,
        tv_target: Option<Utf8PathBuf>,
    ) -> Self {
        Self {
            current_screen: Screen::Dashboard,
            should_quit: false,
            show_confirmation: false,

            source_path,
            movies_target,
            tv_target,
            dry_run: config.behavior.dry_run,
            conflict_policy: conflict_policy_from_config(&config.behavior.conflict_policy),
            max_workers: config.scanning.concurrency_local,
            follow_symlinks: config.scanning.follow_symlinks,
            confidence_threshold: config.identification.confidence_thresholds.yellow,
            confidence_thresholds: config.identification.confidence_thresholds.clone(),
            operation_status: "Ready".to_string(),
            operation_detail: "Press s to scan, p to plan, m to move.".to_string(),
            activity_tick: 0,

            scan_results: Vec::new(),
            scan_running: false,
            scan_error: None,
            scan_complete: false,
            scan_progress: (0, 0),

            identified_items: Vec::new(),
            filtered_items: Vec::new(),
            search_query: String::new(),
            sort_column: SortColumn::Confidence,
            sort_direction: SortDirection::Desc,
            plan_running: false,
            plan_progress: (0, 0),

            selected_index: 0,
            transfer_queue: Vec::new(),
            transfer_running: false,
            transfer_progress: (0, 0),
            transfer_success_count: 0,

            log_messages: Vec::new(),
            filter_input_active: false,
        }
    }

    pub fn add_log(&mut self, msg: impl Into<String>) {
        self.log_messages.push(msg.into());
        if self.log_messages.len() > 1000 {
            self.log_messages.remove(0);
        }
    }

    pub fn is_busy(&self) -> bool {
        self.scan_running || self.plan_running || self.transfer_running
    }

    pub fn set_operation(&mut self, status: impl Into<String>, detail: impl Into<String>) {
        self.operation_status = status.into();
        self.operation_detail = detail.into();
    }

    pub fn tick(&mut self) {
        self.activity_tick = self.activity_tick.wrapping_add(1);
    }

    pub fn rebuild_filtered(&mut self) {
        self.filtered_items = if self.search_query.is_empty() {
            (0..self.identified_items.len()).collect()
        } else {
            let query = self.search_query.to_lowercase();
            self.identified_items
                .iter()
                .enumerate()
                .filter(|(_, item)| {
                    item.media_item
                        .title
                        .as_ref()
                        .map(|t| t.to_lowercase().contains(&query))
                        .unwrap_or(false)
                })
                .map(|(i, _)| i)
                .collect()
        };

        self.sort_filtered();
    }

    fn sort_filtered(&mut self) {
        let items = &self.identified_items;
        let is_desc = self.sort_direction == SortDirection::Desc;

        self.filtered_items.sort_by(|a, b| {
            let ia = &items[*a];
            let ib = &items[*b];
            let cmp = match self.sort_column {
                SortColumn::Title => ia
                    .media_item
                    .title
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .cmp(&ib.media_item.title.as_deref().unwrap_or("").to_lowercase()),
                SortColumn::Year => ia.media_item.year.cmp(&ib.media_item.year),
                SortColumn::Confidence => ia.score.confidence.cmp(&ib.score.confidence),
                SortColumn::Kind => {
                    let kind_a = match ia.media_item.kind {
                        rosey_core::MediaKind::Movie => 0,
                        rosey_core::MediaKind::Show => 1,
                        rosey_core::MediaKind::Episode => 2,
                        rosey_core::MediaKind::Unknown => 3,
                    };
                    let kind_b = match ib.media_item.kind {
                        rosey_core::MediaKind::Movie => 0,
                        rosey_core::MediaKind::Show => 1,
                        rosey_core::MediaKind::Episode => 2,
                        rosey_core::MediaKind::Unknown => 3,
                    };
                    kind_a.cmp(&kind_b)
                }
                SortColumn::Destination => ia.destination.cmp(&ib.destination),
            };
            if is_desc {
                cmp.reverse()
            } else {
                cmp
            }
        });
    }

    pub fn toggle_sort(&mut self, column: SortColumn) {
        if self.sort_column == column {
            self.sort_direction = match self.sort_direction {
                SortDirection::Asc => SortDirection::Desc,
                SortDirection::Desc => SortDirection::Asc,
            };
        } else {
            self.sort_column = column;
            self.sort_direction = SortDirection::Asc;
        }
        self.sort_filtered();
    }

    pub fn next_conflict_policy(&mut self) {
        self.conflict_policy = match self.conflict_policy {
            ConflictPolicy::Skip => ConflictPolicy::Replace,
            ConflictPolicy::Replace => ConflictPolicy::KeepBoth,
            ConflictPolicy::KeepBoth => ConflictPolicy::Skip,
        };
    }

    pub fn get_conflict_policy_name(&self) -> &'static str {
        match self.conflict_policy {
            ConflictPolicy::Skip => "Skip",
            ConflictPolicy::Replace => "Replace",
            ConflictPolicy::KeepBoth => "Keep Both",
        }
    }
}

fn conflict_policy_from_config(policy: &str) -> ConflictPolicy {
    match policy {
        "replace" => ConflictPolicy::Replace,
        "keep_both" => ConflictPolicy::KeepBoth,
        _ => ConflictPolicy::Skip,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_state_uses_config_defaults() {
        let mut config = RoseyConfig::default();
        config.paths.source = "/config/source".into();
        config.paths.movies = "/config/movies".into();
        config.paths.tv = "/config/tv".into();
        config.behavior.dry_run = false;
        config.behavior.conflict_policy = "replace".into();
        config.scanning.concurrency_local = 14;
        config.identification.confidence_thresholds.green = 82;
        config.identification.confidence_thresholds.yellow = 51;

        let app = AppState::new(
            &config,
            Utf8PathBuf::from("/override/source"),
            Some(Utf8PathBuf::from("/override/movies")),
            Some(Utf8PathBuf::from("/override/tv")),
        );

        assert_eq!(app.source_path, Utf8PathBuf::from("/override/source"));
        assert_eq!(app.movies_target, Some(Utf8PathBuf::from("/override/movies")));
        assert_eq!(app.tv_target, Some(Utf8PathBuf::from("/override/tv")));
        assert!(!app.dry_run);
        assert_eq!(app.conflict_policy, ConflictPolicy::Replace);
        assert_eq!(app.max_workers, 14);
        assert_eq!(app.confidence_threshold, 51);
        assert_eq!(app.confidence_thresholds.green, 82);
        assert_eq!(app.confidence_thresholds.yellow, 51);
        assert!(!app.follow_symlinks);
        assert!(!app.is_busy());
    }
}
