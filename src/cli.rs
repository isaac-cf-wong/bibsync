//! Shared command-line runner for the native binary and Python entry point.

use crate::{ProviderChoice, SyncOptions, UpdateMode, pre_commit_hook_manifest, sync_files};
use clap::{Parser, ValueEnum};
use std::ffi::OsString;
use std::path::PathBuf;

/// Synchronize a BibTeX file from TeX citation keys.
#[derive(Debug, Parser)]
#[command(version, about)]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    /// TeX or Markdown files to scan, or a single BibTeX file to check or update.
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

    /// Skip all existing entries without re-resolving them.
    ///
    /// By default only preprint entries are re-checked for publication.
    #[arg(long = "no-update", conflicts_with = "update_all")]
    no_update: bool,

    /// Re-resolve all existing entries from the provider.
    #[arg(long = "update-all", conflicts_with = "no_update")]
    update_all: bool,

    /// Regenerate all existing entries from the provider, overriding --no-update.
    #[arg(long)]
    force_regenerate: bool,

    /// Path to a file listing citekeys to skip, one per line (supports # comments).
    #[arg(long = "ignore-file", value_name = "FILE")]
    ignore_file: Option<PathBuf>,

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

/// Run the command-line interface with process arguments.
#[must_use]
pub fn run_cli() -> i32 {
    run_cli_from(std::env::args_os())
}

/// Run the command-line interface with explicit arguments.
#[must_use]
pub fn run_cli_from<I, T>(args: I) -> i32
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return error.exit_code();
        }
    };
    if cli.print_pre_commit_hook {
        print!(
            "{manifest}",
            manifest = pre_commit_hook_manifest(cli.ignore_file.as_deref())
        );
        return 0;
    }
    if cli.files.is_empty() {
        eprintln!("error: at least one TeX file, Markdown file, or BibTeX file is required");
        return 2;
    }

    let update_mode = if cli.no_update {
        UpdateMode::Never
    } else if cli.update_all {
        UpdateMode::Always
    } else {
        UpdateMode::PreprinsOnly
    };

    let options = SyncOptions {
        output: cli.output,
        other_bibliographies: cli.other_bibliographies,
        provider: cli.provider.into(),
        update_mode,
        force_regenerate: cli.force_regenerate,
        merge_other: cli.merge_other,
        backup: !cli.no_backup,
        check: cli.check || !cli.fix,
        cache: cli.cache,
        refresh_cache: cli.refresh_cache,
        cache_dir: cli.cache_dir,
        ignore_file: cli.ignore_file,
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
                println!("unresolved:");
                for detail in &report.unresolved_details {
                    println!("  {}: {}", detail.key, detail.reason);
                }
            }
            if report.changed {
                if report.check_mode {
                    eprintln!(
                        "bibsync: bibliography is out of date; rerun with --fix to update it"
                    );
                    return 1;
                }
                println!("wrote updated bibliography");
            } else {
                println!("bibliography is up to date");
            }
            i32::from(!report.unresolved.is_empty())
        }
        Err(error) => {
            eprintln!("error: {error}");
            2
        }
    }
}
