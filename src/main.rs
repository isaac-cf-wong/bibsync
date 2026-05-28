//! Command-line entry point for `bibsync`.

use bibsync::{ProviderChoice, SyncOptions, pre_commit_hook_manifest, sync_files};
use clap::{Parser, ValueEnum};
use std::path::PathBuf;

/// Synchronize a BibTeX file from TeX citation keys.
#[derive(Debug, Parser)]
#[command(version, about)]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    /// TeX files to scan, or a single BibTeX file to check or update.
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Main BibTeX file to check or update.
    #[arg(short, long, value_name = "BIB")]
    output: Option<PathBuf>,

    /// Read-only BibTeX files containing existing references.
    #[arg(short = 'r', long = "other", value_name = "BIB")]
    other_bibliographies: Vec<PathBuf>,

    /// Bibliography provider to use.
    #[arg(long, value_enum, default_value_t = CliProvider::Auto)]
    provider: CliProvider,

    /// Do not check existing entries for updates.
    #[arg(long = "no-update")]
    no_update: bool,

    /// Regenerate existing entries when a provider can resolve them.
    #[arg(long)]
    force_regenerate: bool,

    /// Merge entries found in read-only bibliography files.
    #[arg(long)]
    merge_other: bool,

    /// Do not create a .bak file before overwriting the output.
    #[arg(long = "no-backup")]
    no_backup: bool,

    /// Check whether the BibTeX file is current without writing changes (default).
    #[arg(long)]
    check: bool,

    /// Update the BibTeX file instead of only checking it.
    #[arg(long, conflicts_with = "check")]
    fix: bool,

    /// Cache provider responses and reuse cached entries.
    #[arg(long)]
    cache: bool,

    /// Refresh provider responses and update the cache.
    #[arg(long)]
    refresh_cache: bool,

    /// Override the cache directory.
    #[arg(long, value_name = "DIR")]
    cache_dir: Option<PathBuf>,

    /// Print a pre-commit hook manifest for this package.
    #[arg(long)]
    print_pre_commit_hook: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum CliProvider {
    Auto,
    Ads,
    Inspire,
}

impl From<CliProvider> for ProviderChoice {
    fn from(value: CliProvider) -> Self {
        match value {
            CliProvider::Auto => Self::Auto,
            CliProvider::Ads => Self::Ads,
            CliProvider::Inspire => Self::Inspire,
        }
    }
}

fn main() {
    let cli = Cli::parse();
    if cli.print_pre_commit_hook {
        print!("{manifest}", manifest = pre_commit_hook_manifest());
        return;
    }
    if cli.files.is_empty() {
        eprintln!("error: at least one TeX file or one BibTeX file is required");
        std::process::exit(2);
    }

    let options = SyncOptions {
        output: cli.output,
        other_bibliographies: cli.other_bibliographies,
        provider: cli.provider.into(),
        update_existing: !cli.no_update,
        force_regenerate: cli.force_regenerate,
        merge_other: cli.merge_other,
        backup: !cli.no_backup,
        check: cli.check || !cli.fix,
        cache: cli.cache,
        refresh_cache: cli.refresh_cache,
        cache_dir: cli.cache_dir,
    };

    match sync_files(&cli.files, &options) {
        Ok(report) => {
            println!("output: {}", report.output.display());
            if !report.added.is_empty() {
                println!("added: {}", report.added.join(", "));
            }
            if !report.updated.is_empty() {
                println!("updated: {}", report.updated.join(", "));
            }
            if !report.found_in_other.is_empty() {
                println!(
                    "found in other bibliography: {}",
                    report.found_in_other.join(", ")
                );
            }
            if !report.unresolved.is_empty() {
                println!("unresolved: {}", report.unresolved.join(", "));
            }
            if report.changed {
                if report.check_mode {
                    eprintln!(
                        "bibsync: bibliography is out of date; rerun with --fix to update it"
                    );
                    std::process::exit(1);
                }
                println!("wrote updated bibliography");
            } else {
                println!("bibliography is up to date");
            }
            if !report.unresolved.is_empty() {
                std::process::exit(1);
            }
        }
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(2);
        }
    }
}
