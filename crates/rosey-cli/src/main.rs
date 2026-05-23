use anyhow::Result;
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand};
use rosey_core::{extract_episode_info, extract_year};
use rosey_fs::{scan, ScanOptions};
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
    /// Scan a directory for video files.
    Scan {
        root: Utf8PathBuf,

        #[arg(long)]
        json: bool,
    },

    /// Run early offline filename identification helpers.
    Identify {
        path: Utf8PathBuf,

        #[arg(long)]
        json: bool,
    },
}

#[derive(Debug, Serialize)]
struct IdentifyOutput {
    path: Utf8PathBuf,
    year: Option<u16>,
    episode: Option<rosey_core::EpisodeMatch>,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter("warn").init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Scan { root, json } => {
            let results = scan(&root, ScanOptions::default());
            if json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                for result in results {
                    println!("{} ({})", result.path, result.size_bytes);
                }
            }
        }
        Commands::Identify { path, json } => {
            let filename = path.file_name().unwrap_or(path.as_str());
            let output = IdentifyOutput {
                path,
                year: extract_year(filename),
                episode: extract_episode_info(filename),
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!("{output:#?}");
            }
        }
    }

    Ok(())
}
