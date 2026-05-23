use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Parser, Subcommand, ValueEnum};
use rosey_core::{
    clean_title_with_year, confidence_band, discover_companion_files, find_nfo_for_file, parse_nfo,
    ConfidenceBand, ConflictPolicy, MediaItem, MediaKind, Score,
};
use rosey_fs::{move_with_sidecars, Scanner};
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Parser)]
#[command(name = "rosey")]
#[command(about = "Rosey Rust CLI and migration parity harness")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Scan a directory for video files.
    Scan {
        root: Utf8PathBuf,

        #[arg(long)]
        json: bool,

        #[arg(long, default_value = "8")]
        max_workers: usize,
    },

    /// Run offline filename identification on a single file.
    Identify {
        path: Utf8PathBuf,

        #[arg(long)]
        json: bool,
    },

    /// Scan, identify, plan, and optionally move files.
    Run {
        /// Source directory to scan.
        source: Utf8PathBuf,

        /// Target directory for movies.
        #[arg(long)]
        movies_target: Option<Utf8PathBuf>,

        /// Target directory for TV shows.
        #[arg(long)]
        tv_target: Option<Utf8PathBuf>,

        /// Dry-run mode (default).
        #[arg(long, default_value = "true")]
        dry_run: bool,

        /// Disable dry-run and execute live moves.
        #[arg(long)]
        no_dry_run: bool,

        /// Maximum concurrent workers for scanning.
        #[arg(long, default_value = "8")]
        max_workers: usize,

        /// Minimum confidence threshold to display (0-100).
        #[arg(long, default_value = "0")]
        confidence: u8,

        /// Conflict policy for live moves.
        #[arg(long, value_enum, default_value = "skip")]
        conflict_policy: ConflictPolicyArg,

        /// Output results as JSON.
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
            let output = identify_single_file(&path);
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
                let item = identify_file(&scan_result.path);
                let score = score_identification(&item);

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

/// Identify a single media file from its path.
fn identify_file(path: &Utf8Path) -> MediaItem {
    let filename = path.file_name().unwrap_or(path.as_str());
    let folder_name = path.parent().and_then(|p| p.file_name()).unwrap_or("").to_string();

    // Try NFO
    let nfo_data = find_nfo_for_file(path).and_then(|p| parse_nfo(&p));

    // Try companion files
    let companions = discover_companion_files(path);

    // Extract year
    let year = rosey_core::extract_year(filename).or_else(|| {
        if !folder_name.is_empty() {
            rosey_core::extract_year(&folder_name)
        } else {
            None
        }
    });

    // Try to extract episode info
    let episode_info = rosey_core::extract_episode_info(filename, None).or_else(|| {
        if !folder_name.is_empty() {
            rosey_core::extract_episode_info(&folder_name, None)
        } else {
            None
        }
    });

    // Extract date
    let date = rosey_core::extract_date(filename).map(|d| d.date);

    // Extract part
    let part = rosey_core::extract_part(filename);

    // Determine kind and build item
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

    // Enhance title from folder if still unknown
    if item.title.is_none() && !folder_name.is_empty() {
        let folder_year = rosey_core::extract_year(&folder_name);
        item.title = Some(clean_title_with_year(&folder_name, folder_year));
    }

    item
}

/// Score an identification result.
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

    // Cap at 100
    confidence = confidence.min(100);

    Score { confidence, reasons }
}

fn identify_single_file(path: &Utf8Path) -> IdentifyOutput {
    let item = identify_file(path);
    let nfo_path = find_nfo_for_file(path);

    IdentifyOutput {
        path: path.to_path_buf(),
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
    }
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
