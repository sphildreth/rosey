use crate::theme::Theme;
use camino::{Utf8Path, Utf8PathBuf};
use rosey_core::{
    build_tv_shows, discover_show_assets, identify_file_fast, plan_path, run_doctor,
    score_identification, ConfidenceThresholds, ConflictPolicy, DoctorReport, MediaItem, MediaKind,
    Planner, RoseyConfig, Score, TvShow,
};
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
    Doctor,
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
            Screen::Doctor,
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
            Screen::Doctor => "Doctor",
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
            Screen::Doctor => "7",
            Screen::Help => "8",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifiedItem {
    pub media_item: MediaItem,
    pub score: Score,
    pub destination: Utf8PathBuf,
}

impl IdentifiedItem {
    pub fn destination_directory(&self) -> Option<&Utf8Path> {
        if self.destination == self.media_item.source_path {
            return None;
        }
        self.destination.parent()
    }

    pub fn destination_file_exists(&self) -> bool {
        self.destination != self.media_item.source_path && self.destination.exists()
    }

    pub fn destination_directory_exists(&self) -> bool {
        self.destination_directory().is_some_and(Utf8Path::exists)
    }

    pub fn duplicate_indicator(&self) -> Option<&'static str> {
        if self.destination_file_exists() {
            Some("FILE")
        } else if self.destination_directory_exists() {
            Some("DIR")
        } else {
            None
        }
    }

    pub fn is_likely_duplicate(&self) -> bool {
        self.duplicate_indicator().is_some()
    }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualField {
    Kind,
    Title,
    Year,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    Source,
    Movies,
    Tv,
    DryRun,
    ConfirmDelete,
    Theme,
    FollowSymlinks,
    ConflictPolicy,
    MaxWorkers,
    ConfidenceYellow,
    ConfidenceGreen,
    TitleRemoveSegments,
    OnlineProviders,
    TmdbApiKey,
    TmdbLanguage,
    TmdbRegion,
    TvdbApiKey,
    TvdbLanguage,
    CacheTtlDays,
    AutoDeletePatterns,
}

impl SettingsField {
    pub fn all() -> &'static [SettingsField] {
        &[
            SettingsField::Source,
            SettingsField::Movies,
            SettingsField::Tv,
            SettingsField::DryRun,
            SettingsField::ConfirmDelete,
            SettingsField::Theme,
            SettingsField::FollowSymlinks,
            SettingsField::ConflictPolicy,
            SettingsField::MaxWorkers,
            SettingsField::ConfidenceYellow,
            SettingsField::ConfidenceGreen,
            SettingsField::TitleRemoveSegments,
            SettingsField::OnlineProviders,
            SettingsField::TmdbApiKey,
            SettingsField::TmdbLanguage,
            SettingsField::TmdbRegion,
            SettingsField::TvdbApiKey,
            SettingsField::TvdbLanguage,
            SettingsField::CacheTtlDays,
            SettingsField::AutoDeletePatterns,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            SettingsField::Source => "Source",
            SettingsField::Movies => "Movies Target",
            SettingsField::Tv => "TV Target",
            SettingsField::DryRun => "Dry-run",
            SettingsField::ConfirmDelete => "Confirm Delete",
            SettingsField::Theme => "Theme",
            SettingsField::FollowSymlinks => "Follow Symlinks",
            SettingsField::ConflictPolicy => "Conflict Policy",
            SettingsField::MaxWorkers => "Max Workers",
            SettingsField::ConfidenceYellow => "Yellow Threshold",
            SettingsField::ConfidenceGreen => "Green Threshold",
            SettingsField::TitleRemoveSegments => "Title Remove Segments",
            SettingsField::OnlineProviders => "Online Providers",
            SettingsField::TmdbApiKey => "TMDB API Key",
            SettingsField::TmdbLanguage => "TMDB Language",
            SettingsField::TmdbRegion => "TMDB Region",
            SettingsField::TvdbApiKey => "TVDB API Key",
            SettingsField::TvdbLanguage => "TVDB Language",
            SettingsField::CacheTtlDays => "Cache TTL Days",
            SettingsField::AutoDeletePatterns => "Auto-delete Patterns",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettingsEditState {
    pub field: SettingsField,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct ManualProviderResult {
    pub provider: String,
    pub id: String,
    pub kind: MediaKind,
    pub title: String,
    pub year: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManualEditTarget {
    Planned { item_index: usize },
    Scanned { scan_index: usize },
}

#[derive(Debug, Clone)]
pub struct ManualEditState {
    pub target: ManualEditTarget,
    pub field: ManualField,
    pub kind: MediaKind,
    pub title: String,
    pub year: String,
    pub provider_results: Vec<ManualProviderResult>,
    pub provider_index: usize,
    pub search_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingDelete {
    PlanItem { item_index: usize },
    ScanDirectory { directory: Utf8PathBuf },
}

pub struct AppState {
    pub config: RoseyConfig,
    pub current_screen: Screen,
    pub should_quit: bool,
    pub show_confirmation: bool,
    pub pending_delete: Option<PendingDelete>,

    pub source_path: Utf8PathBuf,
    pub movies_target: Option<Utf8PathBuf>,
    pub tv_target: Option<Utf8PathBuf>,
    pub dry_run: bool,
    pub conflict_policy_ask: bool,
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
    pub selected_scan_index: usize,
    pub plan_after_scan: bool,

    pub identified_items: Vec<IdentifiedItem>,
    pub filtered_items: Vec<usize>,
    pub search_query: String,
    pub sort_column: SortColumn,
    pub sort_direction: SortDirection,
    pub plan_running: bool,
    pub plan_progress: (usize, usize),

    pub tv_shows: Vec<TvShow>,
    pub show_group_mode: bool,
    pub selected_show_index: usize,
    pub expanded_show_index: Option<usize>,

    pub selected_index: usize,
    pub transfer_queue: Vec<TransferItem>,
    pub transfer_running: bool,
    pub transfer_progress: (usize, usize),
    pub transfer_success_count: usize,
    pub last_journal_path: Option<Utf8PathBuf>,
    pub recovery_summary: String,

    pub log_messages: Vec<String>,
    pub filter_input_active: bool,
    pub manual_edit: Option<ManualEditState>,
    pub manual_search_running: bool,
    pub selected_settings_index: usize,
    pub settings_edit: Option<SettingsEditState>,
    pub doctor_report: DoctorReport,
    pub doctor_scroll: usize,
    pub theme: Theme,
}

impl AppState {
    pub fn new(
        config: &RoseyConfig,
        source_path: Utf8PathBuf,
        movies_target: Option<Utf8PathBuf>,
        tv_target: Option<Utf8PathBuf>,
    ) -> Self {
        let mut doctor_config = config.clone();
        doctor_config.paths.source = source_path.to_string();
        doctor_config.paths.movies =
            movies_target.as_ref().map(|path| path.to_string()).unwrap_or_default();
        doctor_config.paths.tv =
            tv_target.as_ref().map(|path| path.to_string()).unwrap_or_default();

        Self {
            config: config.clone(),
            current_screen: Screen::Dashboard,
            should_quit: false,
            show_confirmation: false,
            pending_delete: None,

            source_path,
            movies_target,
            tv_target,
            dry_run: config.behavior.dry_run,
            conflict_policy_ask: config.behavior.conflict_policy == "ask",
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
            selected_scan_index: 0,
            plan_after_scan: false,

            identified_items: Vec::new(),
            filtered_items: Vec::new(),
            search_query: String::new(),
            sort_column: SortColumn::Confidence,
            sort_direction: SortDirection::Desc,
            plan_running: false,
            plan_progress: (0, 0),

            tv_shows: Vec::new(),
            show_group_mode: false,
            selected_show_index: 0,
            expanded_show_index: None,

            selected_index: 0,
            transfer_queue: Vec::new(),
            transfer_running: false,
            transfer_progress: (0, 0),
            transfer_success_count: 0,
            last_journal_path: None,
            recovery_summary: "No move journal inspected yet.".to_string(),

            log_messages: Vec::new(),
            filter_input_active: false,
            manual_edit: None,
            manual_search_running: false,
            selected_settings_index: 0,
            settings_edit: None,
            doctor_report: run_doctor(&doctor_config),
            doctor_scroll: 0,
            theme: Theme::from_config(&config.ui.theme),
        }
    }

    pub fn add_log(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        tracing::debug!(message = %msg, "tui event");
        self.log_messages.push(msg);
        if self.log_messages.len() > 1000 {
            self.log_messages.remove(0);
        }
    }

    pub fn is_busy(&self) -> bool {
        self.scan_running
            || self.plan_running
            || self.transfer_running
            || self.manual_search_running
    }

    pub fn build_tv_shows_from_items(&mut self) {
        let tv_root = self.tv_target.as_ref().map(|p| p.as_str()).unwrap_or("");
        let movies_root = self.movies_target.as_ref().map(|p| p.as_str()).unwrap_or("");
        let planner = Planner {
            movies_root: Utf8PathBuf::from(movies_root),
            tv_root: Utf8PathBuf::from(tv_root),
        };
        let items_with_scores: Vec<(MediaItem, Score)> =
            self.identified_items.iter().map(|i| (i.media_item.clone(), i.score.clone())).collect();

        let mut shows = build_tv_shows(&items_with_scores, &planner);
        for show in &mut shows {
            let assets = discover_show_assets(show);
            show.show_assets = assets;
            show.resolve_asset_destinations(tv_root);
            for season in &mut show.seasons {
                for episode in &mut season.episodes {
                    episode.destination = planner.plan_destination(&episode.item);
                }
            }
        }
        self.tv_shows = shows;
    }

    pub fn toggle_show_group_mode(&mut self) {
        self.show_group_mode = !self.show_group_mode;
        self.selected_show_index = 0;
        self.expanded_show_index = None;
        if self.show_group_mode {
            self.build_tv_shows_from_items();
        }
        self.rebuild_filtered();
    }

    pub fn set_operation(&mut self, status: impl Into<String>, detail: impl Into<String>) {
        let status = status.into();
        let detail = detail.into();
        tracing::debug!(status = %status, detail = %detail, "operation status updated");
        self.operation_status = status;
        self.operation_detail = detail;
    }

    pub fn refresh_doctor(&mut self) {
        self.sync_settings_to_config();
        self.doctor_report = run_doctor(&self.config);
        self.doctor_scroll = 0;
        self.set_operation(
            "Doctor refreshed",
            format!(
                "{} errors, {} warnings",
                self.doctor_report.errors(),
                self.doctor_report.warnings()
            ),
        );
    }

    pub fn doctor_scroll_up(&mut self, amount: usize) {
        self.doctor_scroll = self.doctor_scroll.saturating_sub(amount);
    }

    pub fn doctor_scroll_down(&mut self, amount: usize) {
        let max_scroll = self.doctor_report.checks.len().saturating_mul(3).saturating_add(12);
        self.doctor_scroll = self.doctor_scroll.saturating_add(amount).min(max_scroll);
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

    pub fn scan_previous_item(&mut self) {
        self.selected_scan_index = self.selected_scan_index.saturating_sub(1);
    }

    pub fn scan_next_item(&mut self) {
        let last = self.scan_results.len().saturating_sub(1);
        self.selected_scan_index = (self.selected_scan_index + 1).min(last);
    }

    pub fn request_remove_selected_plan_item(&mut self) -> Result<(), String> {
        let Some(&item_index) = self.filtered_items.get(self.selected_index) else {
            return Err("No planned move is selected.".to_string());
        };

        if self.config.behavior.confirm_delete {
            self.pending_delete = Some(PendingDelete::PlanItem { item_index });
            return Ok(());
        }

        self.remove_plan_item(item_index)
    }

    pub fn request_delete_selected_scan_directory(&mut self) -> Result<(), String> {
        let directory = self.selected_scan_directory_for_delete()?;

        if self.config.behavior.confirm_delete {
            self.pending_delete = Some(PendingDelete::ScanDirectory { directory });
            return Ok(());
        }

        self.delete_scan_directory(directory)
    }

    pub fn confirm_pending_delete(&mut self) -> Result<(), String> {
        let Some(action) = self.pending_delete.take() else {
            return Ok(());
        };

        match action {
            PendingDelete::PlanItem { item_index } => self.remove_plan_item(item_index),
            PendingDelete::ScanDirectory { directory } => self.delete_scan_directory(directory),
        }
    }

    pub fn delete_pending_plan_file_from_disk(&mut self) -> Result<(), String> {
        let Some(action) = self.pending_delete.take() else {
            return Err("No pending plan item delete action.".to_string());
        };

        match action {
            PendingDelete::PlanItem { item_index } => self.delete_plan_item_file(item_index),
            other => {
                self.pending_delete = Some(other);
                Err("Del is only available for pending plan item removal.".to_string())
            }
        }
    }

    pub fn cancel_pending_delete(&mut self) {
        self.pending_delete = None;
    }

    fn selected_scan_directory_for_delete(&self) -> Result<Utf8PathBuf, String> {
        let Some(scan_result) = self.scan_results.get(self.selected_scan_index) else {
            return Err("No scanned file is selected.".to_string());
        };

        if scan_result.error.is_some() || !scan_result.is_video {
            return Err("Select a scanned video file before deleting a directory.".to_string());
        }

        let directory = if scan_result.path.is_dir() {
            scan_result.path.clone()
        } else {
            scan_result
                .path
                .parent()
                .map(Utf8Path::to_path_buf)
                .ok_or_else(|| "Selected scan result has no parent directory.".to_string())?
        };

        if directory == self.source_path {
            return Err("Refusing to delete the configured source directory.".to_string());
        }

        if !self.source_path.as_str().is_empty() && !directory.starts_with(&self.source_path) {
            return Err(
                "Refusing to delete a directory outside the configured source path.".to_string()
            );
        }

        if !directory.exists() {
            return Err(format!("Directory no longer exists: {directory}"));
        }

        if !directory.is_dir() {
            return Err(format!("Selected delete target is not a directory: {directory}"));
        }

        Ok(directory)
    }

    fn remove_plan_item(&mut self, item_index: usize) -> Result<(), String> {
        if item_index >= self.identified_items.len() {
            return Err("Planned item no longer exists.".to_string());
        }

        let item = self.identified_items.remove(item_index);
        let label = item
            .media_item
            .title
            .as_deref()
            .unwrap_or_else(|| item.media_item.source_path.as_str())
            .to_string();
        self.rebuild_filtered();
        self.selected_index = self.selected_index.min(self.filtered_items.len().saturating_sub(1));
        self.set_operation("Plan item removed", format!("Removed {label} from the move plan."));
        self.add_log(format!("Removed planned move: {label}"));
        Ok(())
    }

    fn delete_plan_item_file(&mut self, item_index: usize) -> Result<(), String> {
        let item = self
            .identified_items
            .get(item_index)
            .ok_or_else(|| "Planned item no longer exists.".to_string())?;
        let source_path = item.media_item.source_path.clone();
        let label = item
            .media_item
            .title
            .as_deref()
            .unwrap_or_else(|| item.media_item.source_path.as_str())
            .to_string();

        if !source_path.exists() {
            return Err(format!("Source file no longer exists: {source_path}"));
        }

        if !source_path.is_file() {
            return Err(format!("Source path is not a file: {source_path}"));
        }

        std::fs::remove_file(source_path.as_std_path())
            .map_err(|err| format!("Failed to delete {source_path}: {err}"))?;

        self.identified_items.remove(item_index);
        self.scan_results.retain(|result| result.path != source_path);
        self.selected_scan_index =
            self.selected_scan_index.min(self.scan_results.len().saturating_sub(1));
        self.rebuild_filtered();
        self.selected_index = self.selected_index.min(self.filtered_items.len().saturating_sub(1));
        self.set_operation(
            "Source file deleted",
            format!("Deleted {source_path} and removed {label} from the move plan."),
        );
        self.add_log(format!("Deleted source file and removed planned move: {source_path}"));
        Ok(())
    }

    fn delete_scan_directory(&mut self, directory: Utf8PathBuf) -> Result<(), String> {
        std::fs::remove_dir_all(directory.as_std_path())
            .map_err(|err| format!("Failed to delete {directory}: {err}"))?;

        let before_scan = self.scan_results.len();
        self.scan_results.retain(|result| !path_is_within(&result.path, &directory));
        self.selected_scan_index =
            self.selected_scan_index.min(self.scan_results.len().saturating_sub(1));

        let before_plan = self.identified_items.len();
        self.identified_items
            .retain(|item| !path_is_within(&item.media_item.source_path, &directory));
        self.rebuild_filtered();
        self.selected_index = self.selected_index.min(self.filtered_items.len().saturating_sub(1));

        let removed_scan = before_scan.saturating_sub(self.scan_results.len());
        let removed_plan = before_plan.saturating_sub(self.identified_items.len());
        self.set_operation(
            "Scan directory deleted",
            format!(
                "Deleted {directory}; removed {removed_scan} scan item(s) and {removed_plan} planned move(s)."
            ),
        );
        self.add_log(format!(
            "Deleted scan directory: {directory} ({removed_scan} scan item(s), {removed_plan} planned move(s) removed)"
        ));
        Ok(())
    }

    pub fn begin_manual_edit_from_scan(&mut self) -> Result<(), String> {
        let Some(scan_result) = self.scan_results.get(self.selected_scan_index) else {
            return Err("No scanned file is selected.".to_string());
        };

        if scan_result.error.is_some() || !scan_result.is_video {
            return Err("Select a scanned video file before identifying.".to_string());
        }

        if let Some(item_index) = self
            .identified_items
            .iter()
            .position(|item| item.media_item.source_path == scan_result.path)
        {
            self.begin_manual_edit_for_item(item_index);
            return Ok(());
        }

        let item = identify_file_fast(&scan_result.path, &self.config).item;
        self.manual_edit = Some(ManualEditState {
            target: ManualEditTarget::Scanned { scan_index: self.selected_scan_index },
            field: ManualField::Title,
            kind: item.kind,
            title: item.title.clone().unwrap_or_else(|| {
                scan_result.path.file_stem().map(str::to_string).unwrap_or_default()
            }),
            year: item.year.map(|year| year.to_string()).unwrap_or_default(),
            provider_results: Vec::new(),
            provider_index: 0,
            search_status: "Edit criteria, then press F5 to search providers.".to_string(),
        });
        Ok(())
    }

    pub fn begin_manual_edit(&mut self) {
        let Some(&item_index) = self.filtered_items.get(self.selected_index) else {
            return;
        };
        self.begin_manual_edit_for_item(item_index);
    }

    fn begin_manual_edit_for_item(&mut self, item_index: usize) {
        let item = &self.identified_items[item_index].media_item;
        self.manual_edit = Some(ManualEditState {
            target: ManualEditTarget::Planned { item_index },
            field: ManualField::Title,
            kind: item.kind,
            title: item.title.clone().unwrap_or_default(),
            year: item.year.map(|year| year.to_string()).unwrap_or_default(),
            provider_results: Vec::new(),
            provider_index: 0,
            search_status: "Edit criteria, then press F5 to search providers.".to_string(),
        });
    }

    pub fn manual_next_field(&mut self) {
        if let Some(edit) = &mut self.manual_edit {
            edit.field = match edit.field {
                ManualField::Kind => ManualField::Title,
                ManualField::Title => ManualField::Year,
                ManualField::Year => ManualField::Kind,
            };
        }
    }

    pub fn manual_backspace(&mut self) {
        if let Some(edit) = &mut self.manual_edit {
            match edit.field {
                ManualField::Kind => {}
                ManualField::Title => {
                    edit.title.pop();
                }
                ManualField::Year => {
                    edit.year.pop();
                }
            }
        }
    }

    pub fn manual_push_char(&mut self, c: char) {
        if let Some(edit) = &mut self.manual_edit {
            match edit.field {
                ManualField::Kind => match c.to_ascii_lowercase() {
                    'm' => edit.kind = MediaKind::Movie,
                    'e' | 't' => edit.kind = MediaKind::Episode,
                    's' => edit.kind = MediaKind::Show,
                    'u' => edit.kind = MediaKind::Unknown,
                    _ => {}
                },
                ManualField::Title => edit.title.push(c),
                ManualField::Year => {
                    if c.is_ascii_digit() && edit.year.len() < 4 {
                        edit.year.push(c);
                    }
                }
            }
        }
    }

    pub fn apply_manual_edit(&mut self) {
        let Some(edit) = self.manual_edit.take() else {
            return;
        };
        let Some(mut media_item) = self.media_item_for_manual_target(&edit) else {
            return;
        };

        media_item.kind = edit.kind;
        media_item.title = (!edit.title.trim().is_empty()).then(|| edit.title.trim().to_string());
        media_item.year =
            if edit.year.trim().is_empty() { None } else { edit.year.trim().parse().ok() };
        if media_item.kind == MediaKind::Movie {
            media_item.season = None;
            media_item.episodes.clear();
            media_item.date = None;
        }

        self.upsert_manual_item(edit.target, media_item);
    }

    pub fn manual_select_provider_result(&mut self, delta: isize) {
        let Some(edit) = &mut self.manual_edit else {
            return;
        };
        if edit.provider_results.is_empty() {
            return;
        }

        let len = edit.provider_results.len() as isize;
        let next = (edit.provider_index as isize + delta).clamp(0, len - 1);
        edit.provider_index = next as usize;
    }

    pub fn set_manual_search_status(&mut self, status: impl Into<String>) {
        if let Some(edit) = &mut self.manual_edit {
            edit.search_status = status.into();
        }
    }

    pub fn set_manual_search_results(&mut self, results: Vec<ManualProviderResult>) {
        if let Some(edit) = &mut self.manual_edit {
            let count = results.len();
            edit.provider_results = results;
            edit.provider_index = 0;
            edit.search_status = format!("{count} online result(s) found.");
        }
    }

    pub fn has_manual_provider_selection(&self) -> bool {
        self.manual_edit
            .as_ref()
            .and_then(|edit| edit.provider_results.get(edit.provider_index))
            .is_some()
    }

    pub fn apply_manual_provider_selection(&mut self) {
        let Some(edit) = self.manual_edit.take() else {
            return;
        };
        let Some(provider_result) = edit.provider_results.get(edit.provider_index).cloned() else {
            self.manual_edit = Some(edit);
            return;
        };
        let Some(mut media_item) = self.media_item_for_manual_target(&edit) else {
            return;
        };

        media_item.kind = provider_result.kind;
        media_item.title = Some(provider_result.title.clone());
        media_item.year = provider_result.year;
        if provider_result.kind == MediaKind::Movie {
            media_item.season = None;
            media_item.episodes.clear();
            media_item.date = None;
        }
        let id_key = if provider_result.provider == "tvdb" {
            "tvdbid".to_string()
        } else {
            "tmdbid".to_string()
        };
        media_item.nfo.insert(id_key, Some(provider_result.id.clone()));
        media_item.nfo.insert("_source".to_string(), Some("identification".to_string()));
        media_item.nfo.insert("title".to_string(), Some(provider_result.title.clone()));
        if let Some(year) = provider_result.year {
            media_item.nfo.insert("year".to_string(), Some(year.to_string()));
        }

        self.upsert_manual_item(edit.target, media_item);
    }

    fn media_item_for_manual_target(&self, edit: &ManualEditState) -> Option<MediaItem> {
        match edit.target {
            ManualEditTarget::Planned { item_index } => self
                .identified_items
                .get(item_index)
                .map(|identified| identified.media_item.clone()),
            ManualEditTarget::Scanned { scan_index } => {
                let scan_result = self.scan_results.get(scan_index)?;
                Some(identify_file_fast(&scan_result.path, &self.config).item)
            }
        }
    }

    fn upsert_manual_item(&mut self, target: ManualEditTarget, media_item: MediaItem) {
        let source_path = media_item.source_path.clone();
        let score = score_identification(&media_item);
        let movies_root = self.movies_target.as_ref().map(|path| path.as_str()).unwrap_or("");
        let tv_root = self.tv_target.as_ref().map(|path| path.as_str()).unwrap_or("");
        let destination = plan_path(&media_item, movies_root, tv_root);
        let identified = IdentifiedItem { media_item, score, destination };

        match target {
            ManualEditTarget::Planned { item_index } => {
                if let Some(item) = self.identified_items.get_mut(item_index) {
                    *item = identified;
                } else {
                    self.identified_items.push(identified);
                }
            }
            ManualEditTarget::Scanned { .. } => {
                if let Some(item_index) = self
                    .identified_items
                    .iter()
                    .position(|item| item.media_item.source_path == source_path)
                {
                    self.identified_items[item_index] = identified;
                } else {
                    self.identified_items.push(identified);
                }
            }
        }

        self.rebuild_filtered();
        if let Some(next_selected) = self.filtered_items.iter().position(|&item_index| {
            self.identified_items[item_index].media_item.source_path == source_path
        }) {
            self.selected_index = next_selected;
        } else {
            self.selected_index =
                self.selected_index.min(self.filtered_items.len().saturating_sub(1));
        }
        self.current_screen = Screen::PlanPreview;
    }

    pub fn selected_settings_field(&self) -> SettingsField {
        SettingsField::all()
            .get(self.selected_settings_index)
            .copied()
            .unwrap_or(SettingsField::Source)
    }

    pub fn settings_next_field(&mut self) {
        let last = SettingsField::all().len().saturating_sub(1);
        self.selected_settings_index = (self.selected_settings_index + 1).min(last);
    }

    pub fn settings_previous_field(&mut self) {
        self.selected_settings_index = self.selected_settings_index.saturating_sub(1);
    }

    pub fn begin_settings_edit(&mut self) {
        let field = self.selected_settings_field();
        self.settings_edit = Some(SettingsEditState { field, value: self.settings_value(field) });
    }

    pub fn settings_edit_backspace(&mut self) {
        if let Some(edit) = &mut self.settings_edit {
            edit.value.pop();
        }
    }

    pub fn settings_edit_push_char(&mut self, c: char) {
        if let Some(edit) = &mut self.settings_edit {
            edit.value.push(c);
        }
    }

    pub fn settings_value(&self, field: SettingsField) -> String {
        match field {
            SettingsField::Source => self.source_path.to_string(),
            SettingsField::Movies => {
                self.movies_target.as_ref().map(|p| p.to_string()).unwrap_or_default()
            }
            SettingsField::Tv => self.tv_target.as_ref().map(|p| p.to_string()).unwrap_or_default(),
            SettingsField::DryRun => self.dry_run.to_string(),
            SettingsField::ConfirmDelete => self.config.behavior.confirm_delete.to_string(),
            SettingsField::Theme => self.config.ui.theme.clone(),
            SettingsField::FollowSymlinks => self.follow_symlinks.to_string(),
            SettingsField::ConflictPolicy => {
                if self.conflict_policy_ask {
                    "ask".into()
                } else {
                    conflict_policy_to_config(self.conflict_policy).into()
                }
            }
            SettingsField::MaxWorkers => self.max_workers.to_string(),
            SettingsField::ConfidenceYellow => self.confidence_threshold.to_string(),
            SettingsField::ConfidenceGreen => self.confidence_thresholds.green.to_string(),
            SettingsField::TitleRemoveSegments => {
                self.config.identification.title_remove_segments.join(",")
            }
            SettingsField::OnlineProviders => {
                self.config.identification.use_online_providers.to_string()
            }
            SettingsField::TmdbApiKey => self.config.providers.tmdb_api_key.clone(),
            SettingsField::TmdbLanguage => self.config.providers.tmdb_language.clone(),
            SettingsField::TmdbRegion => self.config.providers.tmdb_region.clone(),
            SettingsField::TvdbApiKey => self.config.providers.tvdb_api_key.clone(),
            SettingsField::TvdbLanguage => self.config.providers.tvdb_language.clone(),
            SettingsField::CacheTtlDays => self.config.providers.cache_ttl_days.to_string(),
            SettingsField::AutoDeletePatterns => {
                self.config.behavior.auto_delete_patterns.join(",")
            }
        }
    }

    pub fn apply_settings_edit(&mut self) -> Result<(), String> {
        let Some(edit) = self.settings_edit.take() else {
            return Ok(());
        };
        let value = edit.value.trim();

        match edit.field {
            SettingsField::Source => self.source_path = Utf8PathBuf::from(value),
            SettingsField::Movies => {
                self.movies_target = (!value.is_empty()).then(|| Utf8PathBuf::from(value));
            }
            SettingsField::Tv => {
                self.tv_target = (!value.is_empty()).then(|| Utf8PathBuf::from(value));
            }
            SettingsField::DryRun => self.dry_run = parse_bool(value)?,
            SettingsField::ConfirmDelete => {
                self.config.behavior.confirm_delete = parse_bool(value)?;
            }
            SettingsField::Theme => {
                self.config.ui.theme = value.to_string();
                self.theme = Theme::from_config(value);
            }
            SettingsField::FollowSymlinks => self.follow_symlinks = parse_bool(value)?,
            SettingsField::ConflictPolicy => {
                let normalized = value.to_ascii_lowercase().replace('-', "_");
                match normalized.as_str() {
                    "ask" => self.conflict_policy_ask = true,
                    "skip" => {
                        self.conflict_policy_ask = false;
                        self.conflict_policy = ConflictPolicy::Skip;
                    }
                    "replace" => {
                        self.conflict_policy_ask = false;
                        self.conflict_policy = ConflictPolicy::Replace;
                    }
                    "keep_both" | "keepboth" => {
                        self.conflict_policy_ask = false;
                        self.conflict_policy = ConflictPolicy::KeepBoth;
                    }
                    _ => return Err("Use ask, skip, replace, or keep_both.".to_string()),
                }
            }
            SettingsField::MaxWorkers => self.max_workers = parse_usize(value)?.max(1),
            SettingsField::ConfidenceYellow => {
                self.confidence_threshold = parse_u8_percent(value)?;
                self.confidence_thresholds.yellow = self.confidence_threshold;
            }
            SettingsField::ConfidenceGreen => {
                self.confidence_thresholds.green = parse_u8_percent(value)?;
            }
            SettingsField::TitleRemoveSegments => {
                self.config.identification.title_remove_segments = value
                    .split(',')
                    .map(str::trim)
                    .filter(|segment| !segment.is_empty())
                    .map(ToString::to_string)
                    .collect();
            }
            SettingsField::OnlineProviders => {
                self.config.identification.use_online_providers = parse_bool(value)?;
            }
            SettingsField::TmdbApiKey => self.config.providers.tmdb_api_key = value.to_string(),
            SettingsField::TmdbLanguage => self.config.providers.tmdb_language = value.to_string(),
            SettingsField::TmdbRegion => self.config.providers.tmdb_region = value.to_string(),
            SettingsField::TvdbApiKey => self.config.providers.tvdb_api_key = value.to_string(),
            SettingsField::TvdbLanguage => self.config.providers.tvdb_language = value.to_string(),
            SettingsField::CacheTtlDays => {
                self.config.providers.cache_ttl_days = value
                    .parse()
                    .map_err(|_| "Cache TTL must be a positive integer.".to_string())?;
            }
            SettingsField::AutoDeletePatterns => {
                self.config.behavior.auto_delete_patterns = value
                    .split(',')
                    .map(str::trim)
                    .filter(|pattern| !pattern.is_empty())
                    .map(ToString::to_string)
                    .collect();
            }
        }

        self.sync_settings_to_config();
        Ok(())
    }

    pub fn next_conflict_policy(&mut self) {
        if self.conflict_policy_ask {
            self.conflict_policy_ask = false;
            self.conflict_policy = ConflictPolicy::Skip;
            return;
        }

        match self.conflict_policy {
            ConflictPolicy::Skip => self.conflict_policy = ConflictPolicy::Replace,
            ConflictPolicy::Replace => self.conflict_policy = ConflictPolicy::KeepBoth,
            ConflictPolicy::KeepBoth => self.conflict_policy_ask = true,
        }
    }

    pub fn sync_settings_to_config(&mut self) {
        self.config.paths.source = self.source_path.to_string();
        self.config.paths.movies =
            self.movies_target.as_ref().map(|p| p.to_string()).unwrap_or_default();
        self.config.paths.tv = self.tv_target.as_ref().map(|p| p.to_string()).unwrap_or_default();
        self.config.behavior.dry_run = self.dry_run;
        self.config.ui.theme = self.config.ui.theme.trim().to_string();
        self.config.behavior.conflict_policy = if self.conflict_policy_ask {
            "ask".to_string()
        } else {
            conflict_policy_to_config(self.conflict_policy).to_string()
        };
        self.config.scanning.concurrency_local = self.max_workers;
        self.config.scanning.follow_symlinks = self.follow_symlinks;
        self.confidence_thresholds.yellow = self.confidence_threshold;
        self.config.identification.confidence_thresholds = self.confidence_thresholds.clone();
    }

    pub fn get_conflict_policy_name(&self) -> &'static str {
        if self.conflict_policy_ask {
            return "Ask";
        }

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

fn conflict_policy_to_config(policy: ConflictPolicy) -> &'static str {
    match policy {
        ConflictPolicy::Skip => "skip",
        ConflictPolicy::Replace => "replace",
        ConflictPolicy::KeepBoth => "keep_both",
    }
}

fn path_is_within(path: &Utf8Path, directory: &Utf8Path) -> bool {
    path == directory || path.starts_with(directory)
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value.to_ascii_lowercase().as_str() {
        "true" | "t" | "yes" | "y" | "1" | "on" => Ok(true),
        "false" | "f" | "no" | "n" | "0" | "off" => Ok(false),
        _ => Err("Use true/false, yes/no, on/off, or 1/0.".to_string()),
    }
}

fn parse_usize(value: &str) -> Result<usize, String> {
    value.parse().map_err(|_| "Value must be a positive integer.".to_string())
}

fn parse_u8_percent(value: &str) -> Result<u8, String> {
    let parsed: u8 = value.parse().map_err(|_| "Value must be 0-100.".to_string())?;
    if parsed <= 100 {
        Ok(parsed)
    } else {
        Err("Value must be 0-100.".to_string())
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
        assert!(!app.conflict_policy_ask);
        assert_eq!(app.conflict_policy, ConflictPolicy::Replace);
        assert_eq!(app.max_workers, 14);
        assert_eq!(app.confidence_threshold, 51);
        assert_eq!(app.confidence_thresholds.green, 82);
        assert_eq!(app.confidence_thresholds.yellow, 51);
        assert!(!app.follow_symlinks);
        assert!(!app.is_busy());
        assert_eq!(app.theme.name, "default");
    }

    #[test]
    fn app_state_uses_configured_theme() {
        let mut config = RoseyConfig::default();
        config.ui.theme = "rainbow".into();

        let app = AppState::new(&config, Utf8PathBuf::from("/source"), None, None);

        assert_eq!(app.theme.name, "rainbow");
    }

    #[test]
    fn sync_settings_to_config_updates_saved_snapshot() {
        let config = RoseyConfig::default();
        let mut app = AppState::new(
            &config,
            Utf8PathBuf::from("/runtime/source"),
            Some(Utf8PathBuf::from("/runtime/movies")),
            Some(Utf8PathBuf::from("/runtime/tv")),
        );
        app.dry_run = false;
        app.conflict_policy_ask = false;
        app.conflict_policy = ConflictPolicy::KeepBoth;
        app.max_workers = 12;
        app.follow_symlinks = true;
        app.confidence_thresholds.green = 88;
        app.confidence_threshold = 57;

        app.sync_settings_to_config();

        assert_eq!(app.config.paths.source, "/runtime/source");
        assert_eq!(app.config.paths.movies, "/runtime/movies");
        assert_eq!(app.config.paths.tv, "/runtime/tv");
        assert!(!app.config.behavior.dry_run);
        assert_eq!(app.config.behavior.conflict_policy, "keep_both");
        assert_eq!(app.config.scanning.concurrency_local, 12);
        assert!(app.config.scanning.follow_symlinks);
        assert_eq!(app.config.identification.confidence_thresholds.green, 88);
        assert_eq!(app.config.identification.confidence_thresholds.yellow, 57);
    }

    #[test]
    fn ask_conflict_policy_round_trips_to_config() {
        let mut config = RoseyConfig::default();
        config.behavior.conflict_policy = "ask".into();
        let mut app = AppState::new(&config, Utf8PathBuf::from("/source"), None, None);

        assert!(app.conflict_policy_ask);
        assert_eq!(app.get_conflict_policy_name(), "Ask");
        app.sync_settings_to_config();
        assert_eq!(app.config.behavior.conflict_policy, "ask");
    }

    #[test]
    fn plan_delete_requires_confirmation_then_removes_item() {
        let config = RoseyConfig::default();
        let mut app = AppState::new(&config, Utf8PathBuf::from("/source"), None, None);
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
            destination: Utf8PathBuf::from("/movies/Movie (2020)/Movie (2020).mkv"),
        });
        app.rebuild_filtered();

        app.request_remove_selected_plan_item().unwrap();

        assert!(matches!(app.pending_delete, Some(PendingDelete::PlanItem { item_index: 0 })));
        assert_eq!(app.identified_items.len(), 1);

        app.confirm_pending_delete().unwrap();

        assert!(app.pending_delete.is_none());
        assert!(app.identified_items.is_empty());
        assert!(app.filtered_items.is_empty());
    }

    #[test]
    fn pending_plan_delete_can_delete_source_file_from_disk() {
        let temp_root = std::env::temp_dir()
            .join(format!("rosey-plan-file-delete-test-{}", std::process::id()));
        std::fs::create_dir_all(&temp_root).unwrap();
        let source_file = temp_root.join("Movie.mkv");
        std::fs::write(&source_file, b"movie").unwrap();
        let source_file = Utf8PathBuf::from_path_buf(source_file).unwrap();

        let config = RoseyConfig::default();
        let mut app = AppState::new(&config, Utf8PathBuf::from("/source"), None, None);
        app.scan_results.push(ScanResult {
            path: source_file.clone(),
            is_video: true,
            size_bytes: 5,
            error: None,
        });
        app.identified_items.push(IdentifiedItem {
            media_item: MediaItem {
                kind: MediaKind::Movie,
                source_path: source_file.clone(),
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
            destination: Utf8PathBuf::from("/movies/Movie (2020)/Movie (2020).mkv"),
        });
        app.rebuild_filtered();
        app.pending_delete = Some(PendingDelete::PlanItem { item_index: 0 });

        app.delete_pending_plan_file_from_disk().unwrap();

        assert!(!source_file.exists());
        assert!(app.pending_delete.is_none());
        assert!(app.identified_items.is_empty());
        assert!(app.scan_results.is_empty());

        let _ = std::fs::remove_dir_all(temp_root);
    }

    #[test]
    fn scan_directory_delete_removes_directory_and_session_items() {
        let temp_root =
            std::env::temp_dir().join(format!("rosey-scan-delete-test-{}", std::process::id()));
        let source = temp_root.join("source");
        let media_dir = source.join("Movie (2020)");
        std::fs::create_dir_all(&media_dir).unwrap();
        let media_file = media_dir.join("Movie.mkv");
        std::fs::write(&media_file, b"movie").unwrap();

        let mut config = RoseyConfig::default();
        config.behavior.confirm_delete = false;
        let source = Utf8PathBuf::from_path_buf(source).unwrap();
        let media_dir_utf8 = Utf8PathBuf::from_path_buf(media_dir).unwrap();
        let media_file = Utf8PathBuf::from_path_buf(media_file).unwrap();
        let mut app = AppState::new(&config, source, None, None);
        app.scan_results.push(ScanResult {
            path: media_file.clone(),
            is_video: true,
            size_bytes: 5,
            error: None,
        });
        app.identified_items.push(IdentifiedItem {
            media_item: MediaItem {
                kind: MediaKind::Movie,
                source_path: media_file,
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
            destination: Utf8PathBuf::from("/movies/Movie (2020)/Movie (2020).mkv"),
        });
        app.rebuild_filtered();

        app.request_delete_selected_scan_directory().unwrap();

        assert!(!media_dir_utf8.exists());
        assert!(app.scan_results.is_empty());
        assert!(app.identified_items.is_empty());

        let _ = std::fs::remove_dir_all(temp_root);
    }

    #[test]
    fn manual_edit_does_not_mark_provider_metadata() {
        let config = RoseyConfig::default();
        let media_item = MediaItem {
            kind: MediaKind::Movie,
            source_path: Utf8PathBuf::from("/source/Old.mkv"),
            title: Some("Old".into()),
            year: Some(2000),
            season: None,
            episodes: Vec::new(),
            part: None,
            date: None,
            sidecars: Vec::new(),
            nfo: Default::default(),
        };
        let mut app = AppState::new(&config, Utf8PathBuf::from("/source"), None, None);
        app.identified_items.push(IdentifiedItem {
            media_item,
            score: Score { confidence: 0, reasons: Vec::new() },
            destination: Utf8PathBuf::from("/old"),
        });
        app.rebuild_filtered();
        app.begin_manual_edit();
        if let Some(edit) = &mut app.manual_edit {
            edit.title = "Correct".into();
            edit.year = "2020".into();
        }

        app.apply_manual_edit();

        let item = &app.identified_items[0].media_item;
        assert_eq!(item.title.as_deref(), Some("Correct"));
        assert_eq!(item.year, Some(2020));
        assert!(!item.nfo.contains_key("_source"));
        assert!(!item.nfo.contains_key("title"));
    }

    #[test]
    fn manual_identify_from_scan_adds_item_to_move_plan() {
        let config = RoseyConfig::default();
        let mut app = AppState::new(
            &config,
            Utf8PathBuf::from("/source"),
            Some(Utf8PathBuf::from("/movies")),
            Some(Utf8PathBuf::from("/tv")),
        );
        app.current_screen = Screen::ScanResults;
        app.scan_results.push(ScanResult {
            path: Utf8PathBuf::from("/source/Old.Name.2000.mkv"),
            is_video: true,
            size_bytes: 1_000,
            error: None,
        });

        app.begin_manual_edit_from_scan().unwrap();
        if let Some(edit) = &mut app.manual_edit {
            edit.kind = MediaKind::Movie;
            edit.title = "Correct Name".into();
            edit.year = "2020".into();
        }
        app.apply_manual_edit();

        assert_eq!(app.current_screen, Screen::PlanPreview);
        assert_eq!(app.identified_items.len(), 1);
        let item = &app.identified_items[0];
        assert_eq!(item.media_item.title.as_deref(), Some("Correct Name"));
        assert_eq!(item.media_item.year, Some(2020));
        assert!(item.destination.as_str().contains("Correct Name (2020)"));
    }

    #[test]
    fn provider_selection_marks_identification_metadata() {
        let config = RoseyConfig::default();
        let media_item = MediaItem {
            kind: MediaKind::Unknown,
            source_path: Utf8PathBuf::from("/source/Old.mkv"),
            title: Some("Old".into()),
            year: None,
            season: Some(1),
            episodes: vec![1],
            part: None,
            date: None,
            sidecars: Vec::new(),
            nfo: Default::default(),
        };
        let mut app = AppState::new(&config, Utf8PathBuf::from("/source"), None, None);
        app.identified_items.push(IdentifiedItem {
            media_item,
            score: Score { confidence: 0, reasons: Vec::new() },
            destination: Utf8PathBuf::from("/old"),
        });
        app.rebuild_filtered();
        app.begin_manual_edit();
        if let Some(edit) = &mut app.manual_edit {
            edit.provider_results.push(ManualProviderResult {
                provider: "tmdb".into(),
                id: "603".into(),
                kind: MediaKind::Movie,
                title: "The Matrix".into(),
                year: Some(1999),
            });
        }

        app.apply_manual_provider_selection();

        let item = &app.identified_items[0].media_item;
        assert_eq!(item.kind, MediaKind::Movie);
        assert_eq!(item.title.as_deref(), Some("The Matrix"));
        assert_eq!(item.year, Some(1999));
        assert_eq!(item.season, None);
        assert!(item.episodes.is_empty());
        assert_eq!(item.nfo.get("tmdbid").and_then(|id| id.as_deref()), Some("603"));
        assert_eq!(
            item.nfo.get("_source").and_then(|source| source.as_deref()),
            Some("identification")
        );
    }

    #[test]
    fn settings_edit_updates_paths_and_provider_config() {
        let config = RoseyConfig::default();
        let mut app = AppState::new(&config, Utf8PathBuf::from("/source"), None, None);

        app.selected_settings_index =
            SettingsField::all().iter().position(|field| *field == SettingsField::Source).unwrap();
        app.begin_settings_edit();
        app.settings_edit.as_mut().unwrap().value = "/new/source".into();
        app.apply_settings_edit().unwrap();

        app.selected_settings_index = SettingsField::all()
            .iter()
            .position(|field| *field == SettingsField::OnlineProviders)
            .unwrap();
        app.begin_settings_edit();
        app.settings_edit.as_mut().unwrap().value = "true".into();
        app.apply_settings_edit().unwrap();

        app.selected_settings_index = SettingsField::all()
            .iter()
            .position(|field| *field == SettingsField::TmdbApiKey)
            .unwrap();
        app.begin_settings_edit();
        app.settings_edit.as_mut().unwrap().value = "key".into();
        app.apply_settings_edit().unwrap();

        app.selected_settings_index =
            SettingsField::all().iter().position(|field| *field == SettingsField::Theme).unwrap();
        app.begin_settings_edit();
        app.settings_edit.as_mut().unwrap().value = "rainbow".into();
        app.apply_settings_edit().unwrap();

        app.selected_settings_index = SettingsField::all()
            .iter()
            .position(|field| *field == SettingsField::TitleRemoveSegments)
            .unwrap();
        app.begin_settings_edit();
        app.settings_edit.as_mut().unwrap().value = "fan edit, commentary".into();
        app.apply_settings_edit().unwrap();

        assert_eq!(app.source_path, Utf8PathBuf::from("/new/source"));
        assert_eq!(app.config.paths.source, "/new/source");
        assert!(app.config.identification.use_online_providers);
        assert_eq!(app.config.providers.tmdb_api_key, "key");
        assert_eq!(app.config.ui.theme, "rainbow");
        assert_eq!(app.theme.name, "rainbow");
        assert_eq!(app.config.identification.title_remove_segments, vec!["fan edit", "commentary"]);
    }

    #[test]
    fn settings_edit_validates_conflict_policy() {
        let config = RoseyConfig::default();
        let mut app = AppState::new(&config, Utf8PathBuf::from("/source"), None, None);
        app.settings_edit =
            Some(SettingsEditState { field: SettingsField::ConflictPolicy, value: "bad".into() });

        let err = app.apply_settings_edit().unwrap_err();

        assert!(err.contains("ask"));
        assert_eq!(app.config.behavior.conflict_policy, "ask");
    }
}
