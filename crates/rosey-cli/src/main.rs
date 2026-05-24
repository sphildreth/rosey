use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand, ValueEnum};
use rosey_core::{confidence_band, ConfidenceBand, ConflictPolicy, MediaItem, MediaKind, Score};
use rosey_fs::{move_with_sidecars, Scanner};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(name = "rosey")]
#[command(about = "Rosey Rust CLI and migration parity harness")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Scan {
        root: Utf8PathBuf,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "8")]
        max_workers: usize,
    },

    Identify {
        path: Utf8PathBuf,
        #[arg(long)]
        json: bool,
    },

    Run {
        source: Utf8PathBuf,
        #[arg(long)]
        movies_target: Option<Utf8PathBuf>,
        #[arg(long)]
        tv_target: Option<Utf8PathBuf>,
        #[arg(long, default_value = "true")]
        dry_run: bool,
        #[arg(long)]
        no_dry_run: bool,
        #[arg(long, default_value = "8")]
        max_workers: usize,
        #[arg(long, default_value = "0")]
        confidence: u8,
        #[arg(long, value_enum, default_value = "skip")]
        conflict_policy: ConflictPolicyArg,
        #[arg(long)]
        json: bool,
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

fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("warn").init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { root, json, max_workers } => {
            let scanner = Scanner::new(max_workers, false);
            let results = scanner.scan(&root);
            if json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                for result in &results {
                    println!("{} ({})", result.path, result.size_bytes);
                }
            }
        }

        Commands::Identify { path, json } => {
            let item = rosey_core::identify_file(&path);
            let nfo_path = rosey_core::find_nfo_for_file(&path);

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
        } => {
            let conflict_policy: ConflictPolicy = conflict_policy.into();
            let actually_dry_run = dry_run && !no_dry_run;

            if !source.exists() {
                eprintln!("Error: Source directory does not exist: {source}");
                std::process::exit(1);
            }

            let scanner = Scanner::new(max_workers, false);
            let scan_results = scanner.scan(&source);
            let video_files: Vec<_> =
                scan_results.into_iter().filter(|r| r.is_video && r.error.is_none()).collect();

            if video_files.is_empty() {
                println!("No video files found in {source}");
                return Ok(());
            }

            let mut results: Vec<RunResultItem> = Vec::new();

            for scan_result in &video_files {
                let item = rosey_core::identify_file(&scan_result.path);
                let score = rosey_core::score_identification(&item);

                if score.confidence < confidence {
                    continue;
                }

                let destination = rosey_core::plan_path(
                    &item,
                    movies_target.as_ref().map(|p| p.as_str()).unwrap_or(""),
                    tv_target.as_ref().map(|p| p.as_str()).unwrap_or(""),
                );

                results.push(RunResultItem { item, score, destination });
            }

            let (green, yellow, red) = partition_by_confidence(&results);

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
                    let label = confidence_label(item.score.confidence);
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
            }
        }
    }

    Ok(())
}

fn partition_by_confidence(
    results: &[RunResultItem],
) -> (Vec<RunResultItem>, Vec<RunResultItem>, Vec<RunResultItem>) {
    let green: Vec<_> = results.iter().filter(|r| r.score.confidence >= 70).cloned().collect();
    let yellow: Vec<_> =
        results.iter().filter(|r| (40..70).contains(&r.score.confidence)).cloned().collect();
    let red: Vec<_> = results.iter().filter(|r| r.score.confidence < 40).cloned().collect();
    (green, yellow, red)
}

fn confidence_label(confidence: u8) -> &'static str {
    match confidence_band(confidence) {
        ConfidenceBand::Green => "GREEN",
        ConfidenceBand::Yellow => "YELLOW",
        ConfidenceBand::Red => "RED",
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
