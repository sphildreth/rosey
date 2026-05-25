use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand, ValueEnum};
use rosey_core::{
    init_fallback_tracing, init_tracing, load_config, run_doctor, save_config as save_rosey_config,
    score_identification_result, ConfidenceBand, ConfidenceThresholds, ConflictPolicy,
    DoctorReport, DoctorStatus, IdentifyOptions, MediaItem, MediaKind, RoseyConfig, Score,
};
use rosey_fs::{format_bytes, move_with_sidecars, Scanner};
use rosey_metadata::identify_file_with_metadata;
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(name = "rosey")]
#[command(about = "Rosey media organizer CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Scan {
        root: Option<Utf8PathBuf>,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        max_workers: Option<usize>,
    },

    Identify {
        path: Utf8PathBuf,
        #[arg(long)]
        json: bool,
    },

    Doctor {
        #[arg(long)]
        json: bool,
    },

    Run {
        source: Option<Utf8PathBuf>,
        #[arg(long)]
        movies_target: Option<Utf8PathBuf>,
        #[arg(long)]
        tv_target: Option<Utf8PathBuf>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        no_dry_run: bool,
        #[arg(long)]
        max_workers: Option<usize>,
        #[arg(long)]
        confidence: Option<u8>,
        #[arg(long, value_enum)]
        conflict_policy: Option<ConflictPolicyArg>,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        save_config: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ConflictPolicyArg {
    Skip,
    Replace,
    KeepBoth,
}

impl From<ConflictPolicyArg> for ConflictPolicy {
    fn from(arg: ConflictPolicyArg) -> Self {
        match arg {
            ConflictPolicyArg::Skip => ConflictPolicy::Skip,
            ConflictPolicyArg::Replace => ConflictPolicy::Replace,
            ConflictPolicyArg::KeepBoth => ConflictPolicy::KeepBoth,
        }
    }
}

#[derive(Debug, Serialize)]
struct IdentifyOutput {
    path: Utf8PathBuf,
    kind: String,
    title: Option<String>,
    year: Option<u16>,
    season: Option<u16>,
    episodes: Vec<u16>,
    part: Option<u16>,
    date: Option<String>,
    tmdb_id: Option<String>,
    nfo_path: Option<Utf8PathBuf>,
}

#[derive(Debug, Clone, Serialize)]
struct RunResultItem {
    item: MediaItem,
    score: Score,
    destination: Utf8PathBuf,
}

#[derive(Debug, Clone, Serialize)]
struct RunOutput {
    green: Vec<RunResultItem>,
    yellow: Vec<RunResultItem>,
    red: Vec<RunResultItem>,
    dry_run: bool,
    moved: usize,
    errors: Vec<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = load_config();
    if let Err(err) = init_tracing(&config) {
        eprintln!("Warning: failed to initialize Rosey logging: {err}");
        init_fallback_tracing();
    }

    tracing::info!(
        config_path = %rosey_core::config_path().display(),
        command = ?cli.command,
        "rosey cli starting"
    );

    match cli.command {
        Commands::Scan { root, json, max_workers } => {
            let Some(root) = resolve_config_path(root, &config.paths.source) else {
                eprintln!("Error: No source directory specified. Provide SOURCE or set paths.source in config.");
                std::process::exit(2);
            };
            let max_workers = resolve_max_workers(max_workers, &config);
            tracing::info!(
                root = %root,
                max_workers,
                follow_symlinks = config.scanning.follow_symlinks,
                json,
                "cli scan started"
            );
            let scanner = Scanner::new(max_workers, config.scanning.follow_symlinks);
            let results = scanner.scan(&root);
            tracing::info!(root = %root, results = results.len(), "cli scan complete");
            for result in &results {
                tracing::debug!(
                    path = %result.path,
                    is_video = result.is_video,
                    size_bytes = result.size_bytes,
                    error = result.error.as_deref().unwrap_or(""),
                    "cli scan result"
                );
            }
            if json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                for result in &results {
                    println!("{} ({})", result.path, format_bytes(result.size_bytes));
                }
            }
        }

        Commands::Identify { path, json } => {
            tracing::info!(path = %path, json, "cli identify started");
            let ident =
                identify_file_with_metadata(&path, &config, IdentifyOptions::default()).await;
            let item = ident.item;
            let nfo_path = rosey_core::find_nfo_for_file(&path);
            tracing::debug!(
                path = %path,
                kind = ?item.kind,
                title = item.title.as_deref().unwrap_or(""),
                year = item.year,
                errors = ident.errors.len(),
                "cli identify complete"
            );

            let output = IdentifyOutput {
                path: path.clone(),
                kind: match item.kind {
                    MediaKind::Movie => "movie".to_string(),
                    MediaKind::Episode => "episode".to_string(),
                    _ => "unknown".to_string(),
                },
                title: item.title,
                year: item.year,
                season: item.season,
                episodes: item.episodes,
                part: item.part,
                date: item.date,
                tmdb_id: item.nfo.get("tmdbid").cloned().flatten(),
                nfo_path,
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("{output:#?}");
            }
        }

        Commands::Doctor { json } => {
            tracing::info!(json, "cli doctor started");
            let report = run_doctor(&config);
            tracing::info!(
                status = ?report.overall,
                errors = report.errors(),
                warnings = report.warnings(),
                "cli doctor complete"
            );
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                print_doctor_report(&report);
            }

            if report.errors() > 0 {
                std::process::exit(1);
            }
        }

        Commands::Run {
            source,
            movies_target,
            tv_target,
            dry_run,
            no_dry_run,
            max_workers,
            confidence,
            conflict_policy,
            json,
            save_config,
        } => {
            let source_cli = source.clone();
            let movies_target_cli = movies_target.clone();
            let tv_target_cli = tv_target.clone();
            let Some(source) = resolve_config_path(source, &config.paths.source) else {
                eprintln!("Error: No source directory specified. Provide SOURCE or set paths.source in config.");
                std::process::exit(2);
            };
            let movies_target = movies_target.or_else(|| optional_path(&config.paths.movies));
            let tv_target = tv_target.or_else(|| optional_path(&config.paths.tv));
            let actually_dry_run = resolve_dry_run(&config, dry_run, no_dry_run);
            let max_workers = resolve_max_workers(max_workers, &config);
            let confidence = resolve_confidence(confidence, &config);
            let conflict_policy =
                resolve_conflict_policy(conflict_policy, &config.behavior.conflict_policy);

            if save_config {
                let config_to_save = apply_cli_paths_to_config(
                    &config,
                    source_cli.as_ref(),
                    movies_target_cli.as_ref(),
                    tv_target_cli.as_ref(),
                );
                save_rosey_config(&config_to_save)?;
            }

            if !source.exists() {
                tracing::error!(source = %source, "cli run source path does not exist");
                eprintln!("Error: Source directory does not exist: {source}");
                std::process::exit(1);
            }

            tracing::info!(
                source = %source,
                movies_target = movies_target.as_ref().map(|p| p.as_str()).unwrap_or(""),
                tv_target = tv_target.as_ref().map(|p| p.as_str()).unwrap_or(""),
                dry_run = actually_dry_run,
                max_workers,
                confidence,
                conflict_policy = ?conflict_policy,
                json,
                "cli run started"
            );

            let scanner = Scanner::new(max_workers, config.scanning.follow_symlinks);
            let scan_results = scanner.scan(&source);
            tracing::debug!(source = %source, results = scan_results.len(), "cli run scan complete");
            let video_files: Vec<_> =
                scan_results.into_iter().filter(|r| r.is_video && r.error.is_none()).collect();
            tracing::debug!(video_files = video_files.len(), "cli run filtered scan results");

            if video_files.is_empty() {
                tracing::info!(source = %source, "cli run found no video files");
                println!("No video files found in {source}");
                return Ok(());
            }

            let mut results: Vec<RunResultItem> = Vec::new();

            for scan_result in &video_files {
                let ident = identify_file_with_metadata(
                    &scan_result.path,
                    &config,
                    IdentifyOptions::default(),
                )
                .await;
                let score = score_identification_result(&ident);
                let item = ident.item;
                tracing::debug!(
                    source = %scan_result.path,
                    kind = ?item.kind,
                    title = item.title.as_deref().unwrap_or(""),
                    year = item.year,
                    confidence = score.confidence,
                    threshold = confidence,
                    errors = ident.errors.len(),
                    "cli run identified candidate"
                );

                if score.confidence < confidence {
                    tracing::debug!(
                        source = %scan_result.path,
                        confidence = score.confidence,
                        threshold = confidence,
                        "cli run skipped candidate below confidence threshold"
                    );
                    continue;
                }

                let destination = rosey_core::plan_path(
                    &item,
                    movies_target.as_ref().map(|p| p.as_str()).unwrap_or(""),
                    tv_target.as_ref().map(|p| p.as_str()).unwrap_or(""),
                );
                tracing::debug!(
                    source = %scan_result.path,
                    destination = %destination,
                    confidence = score.confidence,
                    "cli run planned candidate"
                );

                results.push(RunResultItem { item, score, destination });
            }

            let (green, yellow, red) =
                partition_by_confidence(&results, &config.identification.confidence_thresholds);

            if json {
                let output = RunOutput {
                    green: green.clone(),
                    yellow: yellow.clone(),
                    red: red.clone(),
                    dry_run: actually_dry_run,
                    moved: 0,
                    errors: Vec::new(),
                };
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("Results: {} items", results.len());
                println!("Green: {}, Yellow: {}, Red: {}", green.len(), yellow.len(), red.len());
                println!();

                for item in &results {
                    let label = confidence_label(
                        item.score.confidence,
                        &config.identification.confidence_thresholds,
                    );
                    let info = format_item_info(&item.item);
                    println!("[{label}] {:3}% | {info}", item.score.confidence);
                    println!("  Source: {}", item.item.source_path);
                    println!("  Dest:   {}", item.destination);
                    if !item.score.reasons.is_empty() {
                        println!("  Reasons: {}", item.score.reasons.join("; "));
                    }
                    println!();
                }
            }

            if actually_dry_run {
                tracing::info!(planned = results.len(), "cli run dry-run complete");
                println!("DRY-RUN mode — no files were moved");
            } else {
                let mut moved = 0;
                let mut errors: Vec<String> = Vec::new();

                for result in &results {
                    let move_result = move_with_sidecars(
                        &result.item,
                        &result.destination,
                        conflict_policy,
                        false,
                    );
                    tracing::debug!(
                        source = %result.item.source_path,
                        destination = %result.destination,
                        success = move_result.success,
                        moved = move_result.moved.len(),
                        skipped = move_result.skipped.len(),
                        replaced = move_result.replaced.len(),
                        kept_both = move_result.kept_both.len(),
                        errors = move_result.errors.len(),
                        "cli run move result"
                    );

                    if move_result.success {
                        moved += 1;
                        println!("Moved: {}", result.destination);
                    } else {
                        for err in &move_result.errors {
                            eprintln!("Move error: {err}");
                        }
                        errors.extend(move_result.errors.clone());
                    }
                }

                println!(
                    "Move summary: {moved}/{} succeeded; {} errors",
                    results.len(),
                    errors.len()
                );
                tracing::info!(
                    moved,
                    planned = results.len(),
                    errors = errors.len(),
                    "cli run move complete"
                );
            }
        }
    }

    Ok(())
}

fn partition_by_confidence(
    results: &[RunResultItem],
    thresholds: &ConfidenceThresholds,
) -> (Vec<RunResultItem>, Vec<RunResultItem>, Vec<RunResultItem>) {
    let high = thresholds.green.max(thresholds.yellow);
    let low = thresholds.green.min(thresholds.yellow);
    let green: Vec<_> = results.iter().filter(|r| r.score.confidence >= high).cloned().collect();
    let yellow: Vec<_> =
        results.iter().filter(|r| (low..high).contains(&r.score.confidence)).cloned().collect();
    let red: Vec<_> = results.iter().filter(|r| r.score.confidence < low).cloned().collect();
    (green, yellow, red)
}

fn confidence_label(confidence: u8, thresholds: &ConfidenceThresholds) -> &'static str {
    match configured_confidence_band(confidence, thresholds) {
        ConfidenceBand::Green => "GREEN",
        ConfidenceBand::Yellow => "YELLOW",
        ConfidenceBand::Red => "RED",
    }
}

fn resolve_config_path(value: Option<Utf8PathBuf>, config_value: &str) -> Option<Utf8PathBuf> {
    value.or_else(|| optional_path(config_value))
}

fn optional_path(value: &str) -> Option<Utf8PathBuf> {
    (!value.is_empty()).then(|| Utf8PathBuf::from(value))
}

fn resolve_dry_run(_config: &RoseyConfig, _dry_run: bool, no_dry_run: bool) -> bool {
    !no_dry_run
}

fn resolve_max_workers(value: Option<usize>, config: &RoseyConfig) -> usize {
    value.unwrap_or(config.scanning.concurrency_local)
}

fn resolve_confidence(value: Option<u8>, _config: &RoseyConfig) -> u8 {
    value.unwrap_or(0)
}

fn resolve_conflict_policy(
    override_policy: Option<ConflictPolicyArg>,
    config_policy: &str,
) -> ConflictPolicy {
    override_policy.map(Into::into).unwrap_or_else(|| match config_policy {
        "replace" => ConflictPolicy::Replace,
        "keep_both" => ConflictPolicy::KeepBoth,
        _ => ConflictPolicy::Skip,
    })
}

fn apply_cli_paths_to_config(
    config: &RoseyConfig,
    source: Option<&Utf8PathBuf>,
    movies_target: Option<&Utf8PathBuf>,
    tv_target: Option<&Utf8PathBuf>,
) -> RoseyConfig {
    let mut config = config.clone();

    if let Some(source) = source {
        config.paths.source = source.to_string();
    }
    if let Some(movies_target) = movies_target {
        config.paths.movies = movies_target.to_string();
    }
    if let Some(tv_target) = tv_target {
        config.paths.tv = tv_target.to_string();
    }

    config
}

fn configured_confidence_band(confidence: u8, thresholds: &ConfidenceThresholds) -> ConfidenceBand {
    let high = thresholds.green.max(thresholds.yellow);
    let low = thresholds.green.min(thresholds.yellow);

    if confidence >= high {
        ConfidenceBand::Green
    } else if confidence >= low {
        ConfidenceBand::Yellow
    } else {
        ConfidenceBand::Red
    }
}

fn format_item_info(item: &MediaItem) -> String {
    match item.kind {
        MediaKind::Movie => {
            format!(
                "{} ({})",
                item.title.as_deref().unwrap_or("?"),
                item.year.map(|y| y.to_string()).unwrap_or_else(|| "?".to_string())
            )
        }
        MediaKind::Episode => {
            if let Some(season) = item.season {
                if !item.episodes.is_empty() {
                    let ep_str = format!("S{:02}E{:02}", season, item.episodes[0]);
                    format!("{} - {ep_str}", item.title.as_deref().unwrap_or("?"))
                } else if let Some(ref date) = item.date {
                    format!("{} - {date}", item.title.as_deref().unwrap_or("?"))
                } else {
                    item.title.clone().unwrap_or_else(|| "Unknown".to_string())
                }
            } else if let Some(ref date) = item.date {
                format!("{} - {date}", item.title.as_deref().unwrap_or("?"))
            } else {
                item.title.clone().unwrap_or_else(|| "Unknown".to_string())
            }
        }
        _ => item.title.clone().unwrap_or_else(|| "Unknown".to_string()),
    }
}

fn print_doctor_report(report: &DoctorReport) {
    println!(
        "Rosey Doctor: {} ({} errors, {} warnings)",
        doctor_status_label(report.overall),
        report.errors(),
        report.warnings()
    );
    println!("Config: {}", report.config_path);
    println!();

    for check in &report.checks {
        println!("[{}] {} - {}", doctor_status_label(check.status), check.name, check.message);
        if let Some(detail) = &check.detail {
            println!("      {detail}");
        }
    }
}

fn doctor_status_label(status: DoctorStatus) -> &'static str {
    match status {
        DoctorStatus::Ok => "OK",
        DoctorStatus::Warn => "WARN",
        DoctorStatus::Error => "ERROR",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8Path;

    #[test]
    fn config_defaults_fill_missing_cli_values() {
        let mut config = RoseyConfig::default();
        config.paths.source = "/config/source".into();
        config.paths.movies = "/config/movies".into();
        config.paths.tv = "/config/tv".into();
        config.behavior.dry_run = false;
        config.behavior.conflict_policy = "replace".into();
        config.scanning.concurrency_local = 12;
        config.identification.confidence_thresholds.green = 83;

        assert_eq!(
            optional_path(&config.paths.movies).as_deref(),
            Some(Utf8Path::new("/config/movies"))
        );
        assert_eq!(optional_path(&config.paths.tv).as_deref(), Some(Utf8Path::new("/config/tv")));
        assert_eq!(
            resolve_config_path(None, &config.paths.source).as_deref(),
            Some(Utf8Path::new("/config/source"))
        );
        assert!(resolve_dry_run(&config, false, false));
        assert_eq!(resolve_max_workers(None, &config), 12);
        assert_eq!(resolve_confidence(None, &config), 0);
        assert_eq!(
            resolve_conflict_policy(None, &config.behavior.conflict_policy),
            ConflictPolicy::Replace
        );
        assert_eq!(config.scanning.concurrency_local, 12);
        assert_eq!(config.identification.confidence_thresholds.green, 83);
    }

    #[test]
    fn cli_overrides_config_when_present() {
        let config = RoseyConfig::default();
        assert_eq!(
            resolve_config_path(Some(Utf8PathBuf::from("/cli/source")), &config.paths.source)
                .as_deref(),
            Some(Utf8Path::new("/cli/source"))
        );
        assert!(resolve_dry_run(&config, true, false));
        assert!(!resolve_dry_run(&config, true, true));
        assert_eq!(resolve_max_workers(Some(19), &config), 19);
        assert_eq!(resolve_confidence(Some(61), &config), 61);
        assert_eq!(
            resolve_conflict_policy(
                Some(ConflictPolicyArg::KeepBoth),
                &config.behavior.conflict_policy
            ),
            ConflictPolicy::KeepBoth
        );
        assert_eq!(resolve_conflict_policy(None, "ask"), ConflictPolicy::Skip);
    }

    #[test]
    fn apply_cli_paths_to_config_only_overrides_explicit_values() {
        let mut config = RoseyConfig::default();
        config.paths.source = "/config/source".into();
        config.paths.movies = "/config/movies".into();
        config.paths.tv = "/config/tv".into();

        let updated = apply_cli_paths_to_config(
            &config,
            Some(&Utf8PathBuf::from("/cli/source")),
            None,
            Some(&Utf8PathBuf::from("/cli/tv")),
        );

        assert_eq!(updated.paths.source, "/cli/source");
        assert_eq!(updated.paths.movies, "/config/movies");
        assert_eq!(updated.paths.tv, "/cli/tv");
    }
}
