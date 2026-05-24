use camino::Utf8PathBuf;
use rosey_core::{
    plan_path, run_doctor, score_identification, ConfidenceThresholds, ConflictPolicy,
    DoctorReport, MediaItem, MediaKind, RoseyConfig, Score,
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
    FollowSymlinks,
    ConflictPolicy,
    MaxWorkers,
    ConfidenceYellow,
    ConfidenceGreen,
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
            SettingsField::FollowSymlinks,
            SettingsField::ConflictPolicy,
            SettingsField::MaxWorkers,
            SettingsField::ConfidenceYellow,
            SettingsField::ConfidenceGreen,
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
            SettingsField::FollowSymlinks => "Follow Symlinks",
            SettingsField::ConflictPolicy => "Conflict Policy",
            SettingsField::MaxWorkers => "Max Workers",
            SettingsField::ConfidenceYellow => "Yellow Threshold",
            SettingsField::ConfidenceGreen => "Green Threshold",
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
    pub id: String,
    pub kind: MediaKind,
    pub title: String,
    pub year: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct ManualEditState {
    pub item_index: usize,
    pub field: ManualField,
    pub kind: MediaKind,
    pub title: String,
    pub year: String,
    pub provider_results: Vec<ManualProviderResult>,
    pub provider_index: usize,
    pub search_status: String,
}

pub struct AppState {
    pub config: RoseyConfig,
    pub current_screen: Screen,
    pub should_quit: bool,
    pub show_confirmation: bool,

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
        }
    }

    pub fn add_log(&mut self, msg: impl Into<String>) {
        self.log_messages.push(msg.into());
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

    pub fn set_operation(&mut self, status: impl Into<String>, detail: impl Into<String>) {
        self.operation_status = status.into();
        self.operation_detail = detail.into();
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

    pub fn begin_manual_edit(&mut self) {
        let Some(&item_index) = self.filtered_items.get(self.selected_index) else {
            return;
        };
        let item = &self.identified_items[item_index].media_item;
        self.manual_edit = Some(ManualEditState {
            item_index,
            field: ManualField::Title,
            kind: item.kind,
            title: item.title.clone().unwrap_or_default(),
            year: item.year.map(|year| year.to_string()).unwrap_or_default(),
            provider_results: Vec::new(),
            provider_index: 0,
            search_status: "F5 searches online providers when configured.".to_string(),
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
        let Some(item) = self.identified_items.get_mut(edit.item_index) else {
            return;
        };

        item.media_item.kind = edit.kind;
        item.media_item.title =
            (!edit.title.trim().is_empty()).then(|| edit.title.trim().to_string());
        item.media_item.year =
            if edit.year.trim().is_empty() { None } else { edit.year.trim().parse().ok() };
        item.score = score_identification(&item.media_item);
        let movies_root = self.movies_target.as_ref().map(|path| path.as_str()).unwrap_or("");
        let tv_root = self.tv_target.as_ref().map(|path| path.as_str()).unwrap_or("");
        item.destination = plan_path(&item.media_item, movies_root, tv_root);
        self.rebuild_filtered();
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
        let Some(item) = self.identified_items.get_mut(edit.item_index) else {
            return;
        };

        item.media_item.kind = provider_result.kind;
        item.media_item.title = Some(provider_result.title.clone());
        item.media_item.year = provider_result.year;
        if provider_result.kind == MediaKind::Movie {
            item.media_item.season = None;
            item.media_item.episodes.clear();
            item.media_item.date = None;
        }
        item.media_item.nfo.insert("tmdbid".to_string(), Some(provider_result.id.clone()));
        item.media_item.nfo.insert("_source".to_string(), Some("identification".to_string()));
        item.media_item.nfo.insert("title".to_string(), Some(provider_result.title.clone()));
        if let Some(year) = provider_result.year {
            item.media_item.nfo.insert("year".to_string(), Some(year.to_string()));
        }

        item.score = score_identification(&item.media_item);
        let movies_root = self.movies_target.as_ref().map(|path| path.as_str()).unwrap_or("");
        let tv_root = self.tv_target.as_ref().map(|path| path.as_str()).unwrap_or("");
        item.destination = plan_path(&item.media_item, movies_root, tv_root);
        self.rebuild_filtered();
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

        assert_eq!(app.source_path, Utf8PathBuf::from("/new/source"));
        assert_eq!(app.config.paths.source, "/new/source");
        assert!(app.config.identification.use_online_providers);
        assert_eq!(app.config.providers.tmdb_api_key, "key");
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
