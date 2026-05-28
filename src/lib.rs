//! Synchronize BibTeX files from citation keys in TeX sources.
//!
//! The library scans TeX files for citation keys, resolves identifier-like keys
//! through NASA ADS and/or `InspireHEP`, and merges the resulting BibTeX entries
//! into a target bibliography.

use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;

/// Result alias used by `bibsync`.
pub type Result<T> = std::result::Result<T, BibsyncError>;

/// Errors returned by the sync library.
#[derive(Debug, Error)]
pub enum BibsyncError {
    /// A filesystem operation failed.
    #[error("{path}: {source}")]
    Io {
        /// File path associated with the failure.
        path: PathBuf,
        /// The underlying I/O error.
        source: io::Error,
    },
    /// An HTTP request failed.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    /// NASA ADS was selected without an API token.
    #[error("NASA ADS requires ADS_API_TOKEN")]
    MissingAdsToken,
    /// An HTTP header value could not be built.
    #[error("invalid HTTP header value: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),
    /// JSON cache serialization or parsing failed.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    /// An input file did not point to any BibTeX output file.
    #[error("could not identify a bibliography file; pass --output")]
    MissingOutput,
    /// A provider returned an invalid BibTeX payload.
    #[error("{provider} did not return a usable BibTeX entry for {key}")]
    InvalidProviderBibtex {
        /// Provider name.
        provider: &'static str,
        /// Citekey or identifier being resolved.
        key: String,
    },
}

/// Provider selection for identifier resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderChoice {
    /// Try NASA ADS first, then `InspireHEP`.
    Auto,
    /// Use NASA ADS only.
    Ads,
    /// Use `InspireHEP` only.
    Inspire,
}

/// Controls how existing bibliography entries are handled during synchronization.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UpdateMode {
    /// Re-resolve preprint entries to check whether they have been published;
    /// leave all other existing entries unchanged.
    #[default]
    PreprinsOnly,
    /// Skip all existing entries without re-resolving them.
    Never,
    /// Re-resolve all existing entries from the provider.
    Always,
}

/// Options controlling a synchronization run.
#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools)]
pub struct SyncOptions {
    /// Main bibliography file to update. If absent, it is read from TeX sources.
    pub output: Option<PathBuf>,
    /// Read-only bibliography files used to detect existing entries.
    pub other_bibliographies: Vec<PathBuf>,
    /// Provider selection.
    pub provider: ProviderChoice,
    /// Controls which existing entries are re-resolved.
    pub update_mode: UpdateMode,
    /// Regenerate all existing entries from the provider, overriding `update_mode`.
    pub force_regenerate: bool,
    /// Copy entries from read-only bibliography files into the output file.
    pub merge_other: bool,
    /// Create `<output>.bak` before overwriting an existing output file.
    pub backup: bool,
    /// Compare only; do not write. Pre-commit hooks can use this for validation.
    pub check: bool,
    /// Read and write provider responses from a local cache.
    pub cache: bool,
    /// Ignore cached entries and update the local cache from providers.
    pub refresh_cache: bool,
    /// Override the default cache directory.
    pub cache_dir: Option<PathBuf>,
    /// Path to a file listing citekeys to skip, one per line.
    pub ignore_file: Option<PathBuf>,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            output: None,
            other_bibliographies: Vec::new(),
            provider: ProviderChoice::Auto,
            update_mode: UpdateMode::default(),
            force_regenerate: false,
            merge_other: false,
            backup: true,
            check: false,
            cache: false,
            refresh_cache: false,
            cache_dir: None,
            ignore_file: None,
        }
    }
}

/// Outcome of a synchronization run.
#[derive(Debug, Default)]
pub struct SyncReport {
    /// The bibliography file that was checked or updated.
    pub output: PathBuf,
    /// New citekeys added to the output file.
    pub added: Vec<String>,
    /// Existing citekeys whose entries were regenerated.
    pub updated: Vec<String>,
    /// Citekeys already present in the output file.
    pub existing: Vec<String>,
    /// Citekeys found in secondary bibliography files.
    pub found_in_other: Vec<String>,
    /// Citekeys that could not be resolved.
    pub unresolved: Vec<String>,
    /// Whether the output file content changed.
    pub changed: bool,
    /// Whether changes were only reported because check mode was enabled.
    pub check_mode: bool,
}

/// A resolved BibTeX entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedEntry {
    /// Provider-specific canonical identifier.
    pub canonical_id: String,
    /// BibTeX text for the entry.
    pub bibtex: String,
    /// Provider that produced the entry.
    pub provider: &'static str,
}

/// Interface implemented by bibliography providers.
pub trait BibliographyProvider {
    /// Human-readable provider name.
    fn name(&self) -> &'static str;

    /// Resolve one identifier-like citekey.
    ///
    /// Returns `Ok(None)` when the provider has no matching entry.
    ///
    /// # Errors
    ///
    /// Returns an error when the provider request or response handling fails.
    fn resolve(&self, key: &str) -> Result<Option<ResolvedEntry>>;

    /// Resolve several identifier-like citekeys, bypassing any cache layer.
    ///
    /// Used for preprint entries that need a fresh fetch to detect publication.
    /// The default implementation delegates to [`BibliographyProvider::resolve_many`].
    ///
    /// # Errors
    ///
    /// Returns an error when any provider request or response handling fails.
    fn resolve_many_fresh(&self, keys: &[String]) -> Result<BTreeMap<String, ResolvedEntry>> {
        self.resolve_many(keys)
    }

    /// Resolve several identifier-like citekeys.
    ///
    /// Providers can override this to use batch APIs. The default
    /// implementation calls [`BibliographyProvider::resolve`] for each key.
    ///
    /// # Errors
    ///
    /// Returns an error when provider request or response handling fails.
    fn resolve_many(&self, keys: &[String]) -> Result<BTreeMap<String, ResolvedEntry>> {
        let mut resolved = BTreeMap::new();
        for key in keys {
            if let Some(entry) = self.resolve(key)? {
                resolved.insert(key.clone(), entry);
            }
        }
        Ok(resolved)
    }
}

/// Synchronize a bibliography from TeX or BibTeX input files.
///
/// # Errors
///
/// Returns an error when files cannot be read/written, a requested provider
/// cannot be configured, or a provider request fails.
pub fn sync_files(files: &[PathBuf], options: &SyncOptions) -> Result<SyncReport> {
    let cache_config = CacheConfig::from_options(options);
    let provider = ProviderChain::from_choice(options.provider, &cache_config)?;
    sync_files_with_provider(files, options, &provider)
}

/// Synchronize using a caller-provided provider implementation.
///
/// This is useful for tests and for embedding `bibsync` in tools that already
/// have their own provider/cache layer.
///
/// # Errors
///
/// Returns an error when input/output files cannot be handled or when provider
/// resolution fails.
pub fn sync_files_with_provider(
    files: &[PathBuf],
    options: &SyncOptions,
    provider: &dyn BibliographyProvider,
) -> Result<SyncReport> {
    #![allow(clippy::too_many_lines)]
    let bib_update_mode = files.len() == 1 && has_extension(&files[0], "bib");
    let (keys, output, discovered_other) = if bib_update_mode {
        let output = files[0].clone();
        let bib = Bibliography::read_optional(&output)?;
        (bib.keys(), output, Vec::new())
    } else {
        let tex_scan = scan_tex_files(files)?;
        let output = options
            .output
            .clone()
            .or_else(|| tex_scan.bibliographies.first().cloned())
            .ok_or(BibsyncError::MissingOutput)?;
        let discovered_other = if options.output.is_none() {
            tex_scan.bibliographies.into_iter().skip(1).collect()
        } else {
            Vec::new()
        };
        (tex_scan.citekeys, output, discovered_other)
    };

    let mut other_paths = options.other_bibliographies.clone();
    other_paths.extend(discovered_other);

    let mut report = SyncReport {
        output: output.clone(),
        check_mode: options.check,
        ..SyncReport::default()
    };

    let original = read_to_string_optional(&output)?;
    let mut bibliography = Bibliography::parse(&original);
    let mut other_bibliography = Bibliography::default();
    for path in &other_paths {
        other_bibliography.merge(Bibliography::read_optional(path)?);
    }

    let ignore_set = if let Some(ref path) = options.ignore_file {
        load_ignore_set(path)?
    } else {
        BTreeSet::new()
    };

    let mut to_resolve: Vec<String> = Vec::new();
    // Preprint keys re-checked for publication; bypass the cache layer.
    let mut to_resolve_fresh: Vec<String> = Vec::new();
    let mut key_exists: BTreeMap<String, bool> = BTreeMap::new();

    for key in keys {
        if ignore_set.contains(&key) {
            if bibliography.contains(&key) {
                report.existing.push(key);
            }
            continue;
        }

        let exists = bibliography.contains(&key);
        let exists_in_other = other_bibliography.contains(&key);

        if exists {
            let is_preprint = bibliography.entry(&key).is_some_and(BibEntry::is_preprint);
            let should_resolve = options.force_regenerate
                || match options.update_mode {
                    UpdateMode::Never => false,
                    UpdateMode::PreprinsOnly => is_preprint,
                    UpdateMode::Always => true,
                };
            if !should_resolve {
                report.existing.push(key);
                continue;
            }
        }

        if exists_in_other && !exists {
            if options.merge_other {
                if let Some(entry) = other_bibliography.entry(&key) {
                    bibliography.upsert(entry.clone());
                    report.added.push(key);
                }
            } else {
                report.found_in_other.push(key);
            }
            continue;
        }

        if !is_supported_identifier(&key) {
            if exists {
                report.existing.push(key);
            } else {
                report.unresolved.push(key);
            }
            continue;
        }

        // Route preprint keys through a fresh (cache-bypassing) resolve so we
        // always get an up-to-date publication status from the provider.
        let use_fresh = exists
            && !options.force_regenerate
            && options.update_mode == UpdateMode::PreprinsOnly
            && bibliography.entry(&key).is_some_and(BibEntry::is_preprint);

        key_exists.insert(key.clone(), exists);
        if use_fresh {
            to_resolve_fresh.push(key);
        } else {
            to_resolve.push(key);
        }
    }

    let mut resolved_entries = provider.resolve_many(&to_resolve)?;
    resolved_entries.extend(provider.resolve_many_fresh(&to_resolve_fresh)?);

    let preprint_refresh_keys: BTreeSet<&str> =
        to_resolve_fresh.iter().map(String::as_str).collect();

    for key in to_resolve.iter().chain(to_resolve_fresh.iter()) {
        let exists = key_exists.get(key).copied().unwrap_or(false);
        let Some(resolved) = resolved_entries.get(key) else {
            if exists {
                report.existing.push(key.clone());
            } else {
                report.unresolved.push(key.clone());
            }
            continue;
        };

        let mut entry = BibEntry::parse(&resolved.bibtex).ok_or_else(|| {
            BibsyncError::InvalidProviderBibtex {
                provider: resolved.provider,
                key: key.clone(),
            }
        })?;
        entry.key.clone_from(key);

        // For preprint refresh: skip the upsert if still a preprint — the
        // entry is only updated when the provider confirms it is now published.
        if preprint_refresh_keys.contains(key.as_str()) && entry.is_preprint() {
            report.existing.push(key.clone());
            continue;
        }

        bibliography.upsert(entry);
        if exists {
            report.updated.push(key.clone());
        } else {
            report.added.push(key.clone());
        }
    }

    let new_content = bibliography.to_string();
    report.changed = normalize_newlines(&original) != normalize_newlines(&new_content);
    if report.changed && !options.check {
        if options.backup && output.exists() {
            let backup = output.with_extension(format!(
                "{}.bak",
                output
                    .extension()
                    .and_then(std::ffi::OsStr::to_str)
                    .unwrap_or("bib")
            ));
            fs::copy(&output, &backup).map_err(|source| BibsyncError::Io {
                path: backup,
                source,
            })?;
        }
        fs::write(&output, new_content).map_err(|source| BibsyncError::Io {
            path: output.clone(),
            source,
        })?;
    }

    report.added.sort();
    report.updated.sort();
    report.existing.sort();
    report.found_in_other.sort();
    report.unresolved.sort();
    Ok(report)
}

/// Return citation keys referenced by TeX files.
///
/// # Errors
///
/// Returns an error if a TeX file cannot be read.
pub fn citation_keys(files: &[PathBuf]) -> Result<BTreeSet<String>> {
    Ok(scan_tex_files(files)?.citekeys)
}

#[derive(Debug, Default)]
struct TexScan {
    citekeys: BTreeSet<String>,
    bibliographies: Vec<PathBuf>,
}

fn scan_tex_files(files: &[PathBuf]) -> Result<TexScan> {
    let cite_re = Regex::new(
        r"(?s)\\(?:bibentry|[cC]ite[a-zA-Z]{0,12})\*?\s*(?:[\[<][^\]>]*[\]>]\s*)*\{([^{}]+)\}",
    )
    .expect("valid citation regex");
    let bib_re =
        Regex::new(r"\\(?:no)?bibliography\*?\s*\{([^{}]+)\}").expect("valid bibliography regex");
    let comment_re = Regex::new(r"(?m)(?P<prefix>^|[^\\])%.*$").expect("valid comment regex");
    let mut scan = TexScan::default();

    for file in files {
        let raw = read_to_string(file)?;
        let text = comment_re.replace_all(&raw, "$prefix");
        for captures in cite_re.captures_iter(&text) {
            for key in captures[1]
                .split(',')
                .map(str::trim)
                .filter(|key| !key.is_empty())
            {
                scan.citekeys.insert(key.to_owned());
            }
        }
        for captures in bib_re.captures_iter(&text) {
            for bib in captures[1]
                .split(',')
                .map(str::trim)
                .filter(|bib| !bib.is_empty())
            {
                let mut path = PathBuf::from(bib);
                if path.extension().is_none() {
                    path.set_extension("bib");
                }
                if path.is_relative() {
                    path = file.parent().unwrap_or_else(|| Path::new(".")).join(path);
                }
                scan.bibliographies.push(path);
            }
        }
    }
    Ok(scan)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BibEntry {
    entry_type: String,
    key: String,
    body: String,
}

impl BibEntry {
    fn parse(input: &str) -> Option<Self> {
        let start = input.find('@')?;
        let rest = &input[start + 1..];
        let open = rest.find(['{', '('])?;
        let close_char = if rest.as_bytes().get(open) == Some(&b'{') {
            '}'
        } else {
            ')'
        };
        let entry_type = rest[..open].trim().to_owned();
        let after_open = &rest[open + 1..];
        let comma = after_open.find(',')?;
        let key = after_open[..comma].trim().to_owned();
        let mut depth = 1_i32;
        let mut end = None;
        for (offset, ch) in rest[open + 1..].char_indices() {
            match ch {
                '{' if close_char == '}' => depth += 1,
                '}' if close_char == '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(open + 1 + offset);
                        break;
                    }
                }
                '(' if close_char == ')' => depth += 1,
                ')' if close_char == ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(open + 1 + offset);
                        break;
                    }
                }
                _ => {}
            }
        }
        let body = rest[open + 1 + comma + 1..end?].trim().to_owned();
        if entry_type.is_empty() || key.is_empty() || body.is_empty() {
            return None;
        }
        Some(Self {
            entry_type,
            key,
            body,
        })
    }

    fn render(&self) -> String {
        format!(
            "@{}{{{},\n{}\n}}",
            self.entry_type,
            self.key,
            indent_body(&self.body)
        )
    }

    /// Returns true if the entry appears to be an unpublished preprint.
    ///
    /// Heuristic: the body contains an arXiv archive marker (`archiveprefix`
    /// or `eprinttype`) but no `journal` field, which would indicate the
    /// preprint has since been published in a peer-reviewed venue.
    fn is_preprint(&self) -> bool {
        let body_lower = self.body.to_lowercase();
        (body_lower.contains("archiveprefix") || body_lower.contains("eprinttype"))
            && !body_lower.contains("journal")
    }
}

#[derive(Debug, Default)]
struct Bibliography {
    preamble: String,
    entries: BTreeMap<String, BibEntry>,
}

impl Bibliography {
    fn read_optional(path: &Path) -> Result<Self> {
        Ok(Self::parse(&read_to_string_optional(path)?))
    }

    fn parse(input: &str) -> Self {
        let mut bibliography = Self::default();
        let mut first_entry_start = None;
        for segment in split_bib_entries(input) {
            if first_entry_start.is_none() {
                first_entry_start = input.find(segment);
            }
            if let Some(entry) = BibEntry::parse(segment) {
                bibliography.entries.insert(entry.key.clone(), entry);
            }
        }
        if let Some(index) = first_entry_start {
            input[..index].trim().clone_into(&mut bibliography.preamble);
        } else {
            input.trim().clone_into(&mut bibliography.preamble);
        }
        bibliography
    }

    fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    fn entry(&self, key: &str) -> Option<&BibEntry> {
        self.entries.get(key)
    }

    fn keys(&self) -> BTreeSet<String> {
        self.entries.keys().cloned().collect()
    }

    fn merge(&mut self, other: Self) {
        for entry in other.entries.into_values() {
            self.upsert(entry);
        }
    }

    fn upsert(&mut self, entry: BibEntry) {
        self.entries.insert(entry.key.clone(), entry);
    }
}

impl std::fmt::Display for Bibliography {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.preamble.is_empty() {
            writeln!(formatter, "{}\n", self.preamble.trim())?;
        }
        for (index, entry) in self.entries.values().enumerate() {
            if index > 0 {
                writeln!(formatter)?;
            }
            writeln!(formatter, "{}", entry.render())?;
        }
        Ok(())
    }
}

fn split_bib_entries(input: &str) -> Vec<&str> {
    let mut entries = Vec::new();
    let bytes = input.as_bytes();
    let mut index = 0;
    while let Some(relative_at) = input[index..].find('@') {
        let start = index + relative_at;
        let Some(relative_open) = input[start..].find(['{', '(']) else {
            break;
        };
        let open = start + relative_open;
        let close = if bytes.get(open) == Some(&b'{') {
            b'}'
        } else {
            b')'
        };
        let open_byte = bytes[open];
        let mut depth = 0_i32;
        let mut end = None;
        for (offset, byte) in bytes[open..].iter().enumerate() {
            if *byte == open_byte {
                depth += 1;
            } else if *byte == close {
                depth -= 1;
                if depth == 0 {
                    end = Some(open + offset + 1);
                    break;
                }
            }
        }
        if let Some(end) = end {
            entries.push(&input[start..end]);
            index = end;
        } else {
            break;
        }
    }
    entries
}

fn indent_body(body: &str) -> String {
    body.lines()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else if line.starts_with("  ") {
                line.to_owned()
            } else {
                format!("  {}", line.trim())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn normalize_newlines(input: &str) -> String {
    input.replace("\r\n", "\n").trim().to_owned()
}

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|actual| actual.eq_ignore_ascii_case(extension))
}

fn read_to_string(path: &Path) -> Result<String> {
    fs::read_to_string(path).map_err(|source| BibsyncError::Io {
        path: path.to_owned(),
        source,
    })
}

fn read_to_string_optional(path: &Path) -> Result<String> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(content),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        Err(source) => Err(BibsyncError::Io {
            path: path.to_owned(),
            source,
        }),
    }
}

fn is_supported_identifier(key: &str) -> bool {
    is_arxiv_id(key) || is_doi(key) || is_ads_bibcode(key)
}

fn load_ignore_set(path: &Path) -> Result<BTreeSet<String>> {
    let content = read_to_string_optional(path)?;
    Ok(content
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(String::from)
        .collect())
}

fn is_arxiv_id(key: &str) -> bool {
    let key = key
        .trim()
        .trim_start_matches("arXiv:")
        .trim_start_matches("arxiv:");
    Regex::new(r"^(?:\d{4}\.\d{4,5}(?:v\d+)?|[a-z-]+(?:\.[A-Za-z-]+)?/\d{7}(?:v\d+)?)$")
        .expect("valid arxiv regex")
        .is_match(key)
}

fn normalize_arxiv_id(key: &str) -> String {
    key.trim()
        .trim_start_matches("arXiv:")
        .trim_start_matches("arxiv:")
        .split('v')
        .next()
        .unwrap_or(key)
        .to_owned()
}

fn is_doi(key: &str) -> bool {
    Regex::new(r"^10\.\d{4,}(?:\.\d+)*/\S+$")
        .expect("valid doi regex")
        .is_match(key.trim())
}

fn is_ads_bibcode(key: &str) -> bool {
    Regex::new(r"^\d{4}\D\S{13}[A-Z.:]$")
        .expect("valid bibcode regex")
        .is_match(key.trim())
}

struct ProviderChain {
    providers: Vec<Box<dyn BibliographyProvider>>,
}

impl ProviderChain {
    fn from_choice(choice: ProviderChoice, cache_config: &CacheConfig) -> Result<Self> {
        let providers: Vec<Box<dyn BibliographyProvider>> = match choice {
            ProviderChoice::Auto => {
                let mut providers: Vec<Box<dyn BibliographyProvider>> = Vec::new();
                if let Some(ads) = AdsProvider::from_env_optional()? {
                    providers.push(wrap_provider(Box::new(ads), cache_config));
                }
                providers.push(wrap_provider(
                    Box::new(InspireProvider::new()?),
                    cache_config,
                ));
                providers
            }
            ProviderChoice::Ads => vec![wrap_provider(
                Box::new(AdsProvider::from_env()?),
                cache_config,
            )],
            ProviderChoice::Inspire => vec![wrap_provider(
                Box::new(InspireProvider::new()?),
                cache_config,
            )],
        };
        Ok(Self { providers })
    }
}

impl BibliographyProvider for ProviderChain {
    fn name(&self) -> &'static str {
        "provider chain"
    }

    fn resolve(&self, key: &str) -> Result<Option<ResolvedEntry>> {
        for provider in &self.providers {
            if let Some(entry) = provider.resolve(key)? {
                return Ok(Some(entry));
            }
        }
        Ok(None)
    }

    fn resolve_many(&self, keys: &[String]) -> Result<BTreeMap<String, ResolvedEntry>> {
        let mut resolved = BTreeMap::new();
        let mut remaining = keys.to_vec();
        for provider in &self.providers {
            if remaining.is_empty() {
                break;
            }
            let provider_resolved = provider.resolve_many(&remaining)?;
            remaining.retain(|key| {
                if let Some(entry) = provider_resolved.get(key) {
                    resolved.insert(key.clone(), entry.clone());
                    false
                } else {
                    true
                }
            });
        }
        Ok(resolved)
    }
}

#[derive(Clone, Debug)]
struct CacheConfig {
    enabled: bool,
    refresh: bool,
    root: PathBuf,
}

impl CacheConfig {
    fn from_options(options: &SyncOptions) -> Self {
        let enabled = options.cache || options.refresh_cache;
        Self {
            enabled,
            refresh: options.refresh_cache,
            root: options.cache_dir.clone().unwrap_or_else(default_cache_dir),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct CacheRecord {
    provider: String,
    canonical_id: String,
    bibtex: String,
    fetched_at_unix_seconds: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct CacheMapping {
    provider: String,
    lookup_kind: String,
    lookup_value: String,
    canonical_id: String,
    fetched_at_unix_seconds: u64,
}

struct CachedProvider {
    inner: Box<dyn BibliographyProvider>,
    config: CacheConfig,
}

fn wrap_provider(
    provider: Box<dyn BibliographyProvider>,
    config: &CacheConfig,
) -> Box<dyn BibliographyProvider> {
    if config.enabled {
        Box::new(CachedProvider {
            inner: provider,
            config: config.clone(),
        })
    } else {
        provider
    }
}

impl BibliographyProvider for CachedProvider {
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn resolve(&self, key: &str) -> Result<Option<ResolvedEntry>> {
        let resolved = self.resolve_many(&[key.to_owned()])?;
        Ok(resolved.into_values().next())
    }

    fn resolve_many(&self, keys: &[String]) -> Result<BTreeMap<String, ResolvedEntry>> {
        let provider = self.inner.name();
        let mut resolved = BTreeMap::new();
        let mut misses = Vec::new();

        if self.config.refresh {
            misses.extend_from_slice(keys);
        } else {
            for key in keys {
                if let Some(entry) = cache_lookup(&self.config.root, provider, key)? {
                    resolved.insert(key.clone(), entry);
                } else {
                    misses.push(key.clone());
                }
            }
        }

        if misses.is_empty() {
            return Ok(resolved);
        }

        let fetched = self.inner.resolve_many(&misses)?;
        for (key, entry) in &fetched {
            cache_store(&self.config.root, key, entry)?;
        }
        resolved.extend(fetched);
        Ok(resolved)
    }

    fn resolve_many_fresh(&self, keys: &[String]) -> Result<BTreeMap<String, ResolvedEntry>> {
        let fetched = self.inner.resolve_many(keys)?;
        for (key, entry) in &fetched {
            cache_store(&self.config.root, key, entry)?;
        }
        Ok(fetched)
    }
}

fn cache_lookup(root: &Path, provider: &str, key: &str) -> Result<Option<ResolvedEntry>> {
    let Some((kind, value)) = lookup_parts(key) else {
        return Ok(None);
    };
    let provider_slug = provider_slug(provider);
    let mapping_path = mapping_path(root, provider_slug, kind, &value);
    let Some(mapping) = read_json_optional::<CacheMapping>(&mapping_path)? else {
        return Ok(None);
    };
    let record_path = record_path(root, provider_slug, &mapping.canonical_id);
    let Some(record) = read_json_optional::<CacheRecord>(&record_path)? else {
        return Ok(None);
    };
    Ok(Some(ResolvedEntry {
        canonical_id: record.canonical_id,
        bibtex: record.bibtex,
        provider: provider_name_from_slug(provider_slug),
    }))
}

fn cache_store(root: &Path, key: &str, entry: &ResolvedEntry) -> Result<()> {
    let Some((kind, value)) = lookup_parts(key) else {
        return Ok(());
    };
    let provider_slug = provider_slug(entry.provider);
    let timestamp = unix_timestamp();
    let record = CacheRecord {
        provider: entry.provider.to_owned(),
        canonical_id: entry.canonical_id.clone(),
        bibtex: entry.bibtex.clone(),
        fetched_at_unix_seconds: timestamp,
    };
    write_json(
        &record_path(root, provider_slug, &entry.canonical_id),
        &record,
    )?;

    let mapping = CacheMapping {
        provider: entry.provider.to_owned(),
        lookup_kind: kind.to_owned(),
        lookup_value: value.clone(),
        canonical_id: entry.canonical_id.clone(),
        fetched_at_unix_seconds: timestamp,
    };
    write_json(&mapping_path(root, provider_slug, kind, &value), &mapping)
}

fn read_json_optional<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Option<T>> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(Some(serde_json::from_str(&content)?)),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(source) => Err(BibsyncError::Io {
            path: path.to_owned(),
            source,
        }),
    }
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| BibsyncError::Io {
            path: parent.to_owned(),
            source,
        })?;
    }
    let content = serde_json::to_string_pretty(value)?;
    fs::write(path, format!("{content}\n")).map_err(|source| BibsyncError::Io {
        path: path.to_owned(),
        source,
    })
}

fn lookup_parts(key: &str) -> Option<(&'static str, String)> {
    if is_arxiv_id(key) {
        Some(("arxiv", normalize_arxiv_id(key)))
    } else if is_doi(key) {
        Some(("doi", key.trim().to_ascii_lowercase()))
    } else if is_ads_bibcode(key) {
        Some(("bibcode", key.trim().to_owned()))
    } else {
        None
    }
}

fn mapping_path(root: &Path, provider: &str, kind: &str, value: &str) -> PathBuf {
    root.join(provider)
        .join("mappings")
        .join(kind)
        .join(format!("{}.json", encode_filename(value)))
}

fn record_path(root: &Path, provider: &str, canonical_id: &str) -> PathBuf {
    root.join(provider)
        .join("records")
        .join(format!("{}.json", encode_filename(canonical_id)))
}

fn encode_filename(value: &str) -> String {
    use std::fmt::Write as _;

    value
        .as_bytes()
        .iter()
        .fold(String::new(), |mut encoded, byte| {
            let _ = write!(encoded, "{byte:02x}");
            encoded
        })
}

fn provider_slug(provider: &str) -> &'static str {
    match provider {
        "NASA ADS" => "ads",
        "InspireHEP" => "inspire",
        _ => "provider",
    }
}

fn provider_name_from_slug(slug: &str) -> &'static str {
    match slug {
        "ads" => "NASA ADS",
        "inspire" => "InspireHEP",
        _ => "provider",
    }
}

fn unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn default_cache_dir() -> PathBuf {
    if let Ok(dir) = env::var("BIBSYNC_CACHE_DIR") {
        return PathBuf::from(dir);
    }
    #[cfg(target_os = "windows")]
    {
        if let Ok(dir) = env::var("LOCALAPPDATA") {
            return PathBuf::from(dir).join("bibsync");
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Caches")
                .join("bibsync");
        }
    }
    if let Ok(dir) = env::var("XDG_CACHE_HOME") {
        return PathBuf::from(dir).join("bibsync");
    }
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join(".cache").join("bibsync");
    }
    PathBuf::from(".bibsync-cache")
}

/// NASA ADS bibliography provider.
pub struct AdsProvider {
    client: Client,
    token: String,
}

impl AdsProvider {
    fn from_env_optional() -> Result<Option<Self>> {
        match env::var("ADS_API_TOKEN") {
            Ok(token) => {
                let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
                Ok(Some(Self { client, token }))
            }
            Err(env::VarError::NotPresent) => Ok(None),
            Err(env::VarError::NotUnicode(_)) => Err(BibsyncError::MissingAdsToken),
        }
    }

    /// Build a provider from `ADS_API_TOKEN`.
    ///
    /// # Errors
    ///
    /// Returns an error when the token is missing or the HTTP client cannot be
    /// configured.
    pub fn from_env() -> Result<Self> {
        let token = env::var("ADS_API_TOKEN").map_err(|_| BibsyncError::MissingAdsToken)?;
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
        Ok(Self { client, token })
    }

    fn headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("bibsync/0.1"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.token))?,
        );
        Ok(headers)
    }

    fn bibcode_for_identifier(&self, key: &str) -> Result<Option<String>> {
        if is_ads_bibcode(key) {
            return Ok(Some(key.to_owned()));
        }
        let query = format!("identifier:\"{key}\"");
        let response: AdsSearchResponse = self
            .client
            .get("https://api.adsabs.harvard.edu/v1/search/query")
            .headers(self.headers()?)
            .query(&[("q", query.as_str()), ("fl", "bibcode"), ("rows", "1")])
            .send()?
            .error_for_status()?
            .json()?;
        Ok(response
            .response
            .docs
            .into_iter()
            .find_map(|doc| doc.bibcode))
    }

    fn export_bibtex_many(&self, bibcodes: &[String]) -> Result<BTreeMap<String, String>> {
        let response: AdsExportResponse = self
            .client
            .post("https://api.adsabs.harvard.edu/v1/export/bibtex")
            .headers(self.headers()?)
            .json(&json!({ "bibcode": bibcodes }))
            .send()?
            .error_for_status()?
            .json()?;
        let Some(export) = nonempty(&response.export) else {
            return Ok(BTreeMap::new());
        };
        Ok(split_bib_entries(&export)
            .into_iter()
            .filter_map(|entry| {
                let parsed = BibEntry::parse(entry)?;
                Some((parsed.key.clone(), parsed.render()))
            })
            .collect())
    }
}

impl BibliographyProvider for AdsProvider {
    fn name(&self) -> &'static str {
        "NASA ADS"
    }

    fn resolve(&self, key: &str) -> Result<Option<ResolvedEntry>> {
        let identifier = if is_arxiv_id(key) {
            format!("arXiv:{}", normalize_arxiv_id(key))
        } else {
            key.to_owned()
        };
        let Some(bibcode) = self.bibcode_for_identifier(&identifier)? else {
            return Ok(None);
        };
        let mut exported = self.export_bibtex_many(std::slice::from_ref(&bibcode))?;
        let Some(bibtex) = exported.remove(&bibcode) else {
            return Ok(None);
        };
        Ok(Some(ResolvedEntry {
            canonical_id: bibcode,
            bibtex,
            provider: self.name(),
        }))
    }

    fn resolve_many(&self, keys: &[String]) -> Result<BTreeMap<String, ResolvedEntry>> {
        let mut key_to_bibcode = BTreeMap::new();
        for key in keys {
            let identifier = if is_arxiv_id(key) {
                format!("arXiv:{}", normalize_arxiv_id(key))
            } else {
                key.clone()
            };
            if let Some(bibcode) = self.bibcode_for_identifier(&identifier)? {
                key_to_bibcode.insert(key.clone(), bibcode);
            }
        }

        let bibcodes = key_to_bibcode
            .values()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let exported = self.export_bibtex_many(&bibcodes)?;
        Ok(key_to_bibcode
            .into_iter()
            .filter_map(|(key, bibcode)| {
                let bibtex = exported.get(&bibcode)?.clone();
                Some((
                    key,
                    ResolvedEntry {
                        canonical_id: bibcode,
                        bibtex,
                        provider: self.name(),
                    },
                ))
            })
            .collect())
    }
}

#[derive(Debug, Deserialize)]
struct AdsSearchResponse {
    response: AdsSearchDocs,
}

#[derive(Debug, Deserialize)]
struct AdsSearchDocs {
    docs: Vec<AdsSearchDoc>,
}

#[derive(Debug, Deserialize)]
struct AdsSearchDoc {
    bibcode: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AdsExportResponse {
    export: String,
}

/// `InspireHEP` bibliography provider.
pub struct InspireProvider {
    client: Client,
}

impl InspireProvider {
    /// Build an `InspireHEP` provider.
    ///
    /// # Errors
    ///
    /// Returns an error when the HTTP client cannot be configured.
    pub fn new() -> Result<Self> {
        let client = Client::builder().timeout(Duration::from_secs(30)).build()?;
        Ok(Self { client })
    }
}

impl BibliographyProvider for InspireProvider {
    fn name(&self) -> &'static str {
        "InspireHEP"
    }

    fn resolve(&self, key: &str) -> Result<Option<ResolvedEntry>> {
        let query = if is_arxiv_id(key) {
            format!("arxiv:{}", normalize_arxiv_id(key))
        } else if is_doi(key) {
            format!("doi:{key}")
        } else {
            return Ok(None);
        };
        let bibtex = self
            .client
            .get("https://inspirehep.net/api/literature")
            .header(USER_AGENT, "bibsync/0.1")
            .header(ACCEPT, "application/x-bibtex")
            .query(&[("q", query.as_str()), ("format", "bibtex"), ("size", "1")])
            .send()?
            .error_for_status()?
            .text()?;
        let Some(bibtex) = nonempty(&bibtex) else {
            return Ok(None);
        };
        if !bibtex.trim_start().starts_with('@') {
            return Ok(None);
        }
        Ok(Some(ResolvedEntry {
            canonical_id: query,
            bibtex,
            provider: self.name(),
        }))
    }

    fn resolve_many(&self, keys: &[String]) -> Result<BTreeMap<String, ResolvedEntry>> {
        let query_parts = keys
            .iter()
            .filter_map(|key| {
                if is_arxiv_id(key) {
                    Some(format!("arxiv:{}", normalize_arxiv_id(key)))
                } else if is_doi(key) {
                    Some(format!("doi:{key}"))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        if query_parts.is_empty() {
            return Ok(BTreeMap::new());
        }

        let response: InspireSearchResponse = self
            .client
            .get("https://inspirehep.net/api/literature")
            .header(USER_AGENT, "bibsync/0.1")
            .query(&[
                ("q", query_parts.join(" OR ").as_str()),
                ("size", keys.len().to_string().as_str()),
            ])
            .send()?
            .error_for_status()?
            .json()?;

        let mut record_by_key = BTreeMap::new();
        for hit in response.hits.hits {
            let Some(record_id) = hit
                .id
                .or(hit.metadata.control_number.map(|id| id.to_string()))
            else {
                continue;
            };
            for key in keys {
                if inspire_hit_matches_key(&hit.metadata, key) {
                    record_by_key.insert(key.clone(), record_id.clone());
                }
            }
        }

        let mut resolved = BTreeMap::new();
        for (key, record_id) in record_by_key {
            let bibtex = self
                .client
                .get(format!("https://inspirehep.net/api/literature/{record_id}"))
                .header(USER_AGENT, "bibsync/0.1")
                .header(ACCEPT, "application/x-bibtex")
                .query(&[("format", "bibtex")])
                .send()?
                .error_for_status()?
                .text()?;
            let Some(bibtex) = nonempty(&bibtex) else {
                continue;
            };
            if !bibtex.trim_start().starts_with('@') {
                continue;
            }
            resolved.insert(
                key,
                ResolvedEntry {
                    canonical_id: record_id,
                    bibtex,
                    provider: self.name(),
                },
            );
        }
        Ok(resolved)
    }
}

#[derive(Debug, Deserialize)]
struct InspireSearchResponse {
    hits: InspireHits,
}

#[derive(Debug, Deserialize)]
struct InspireHits {
    hits: Vec<InspireHit>,
}

#[derive(Debug, Deserialize)]
struct InspireHit {
    id: Option<String>,
    metadata: InspireMetadata,
}

#[derive(Debug, Deserialize)]
struct InspireMetadata {
    control_number: Option<u64>,
    #[serde(default)]
    arxiv_eprints: Vec<InspireValue>,
    #[serde(default)]
    dois: Vec<InspireValue>,
}

#[derive(Debug, Deserialize)]
struct InspireValue {
    value: String,
}

fn inspire_hit_matches_key(metadata: &InspireMetadata, key: &str) -> bool {
    if is_arxiv_id(key) {
        let normalized = normalize_arxiv_id(key);
        return metadata
            .arxiv_eprints
            .iter()
            .any(|eprint| normalize_arxiv_id(&eprint.value) == normalized);
    }
    if is_doi(key) {
        return metadata
            .dois
            .iter()
            .any(|doi| doi.value.eq_ignore_ascii_case(key));
    }
    false
}

fn nonempty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

/// Return a pre-commit hook manifest snippet for this repository.
///
/// When `ignore_file` is provided it is appended to the hook `entry` as
/// `--ignore-file <path>` so the hook picks up the ignore list automatically.
#[must_use]
pub fn pre_commit_hook_manifest(ignore_file: Option<&Path>) -> String {
    let entry = if let Some(path) = ignore_file {
        format!("bibsync --ignore-file {}", path.display())
    } else {
        "bibsync".to_owned()
    };
    format!(
        "- id: bibsync\n  name: bibsync\n  description: Synchronize BibTeX entries from TeX citation keys\n  entry: {entry}\n  language: rust\n  types_or: [tex, bib]\n"
    )
}

#[cfg(test)]
mod tests {
    use super::{
        BibliographyProvider, ProviderChoice, ResolvedEntry, SyncOptions, citation_keys,
        sync_files_with_provider,
    };
    use std::cell::Cell;
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use std::rc::Rc;
    use tempfile::tempdir;

    struct FakeProvider {
        entries: BTreeMap<String, String>,
    }

    impl BibliographyProvider for FakeProvider {
        fn name(&self) -> &'static str {
            "fake"
        }

        fn resolve(&self, key: &str) -> super::Result<Option<ResolvedEntry>> {
            Ok(self.entries.get(key).map(|bibtex| ResolvedEntry {
                canonical_id: key.to_owned(),
                bibtex: bibtex.clone(),
                provider: self.name(),
            }))
        }
    }

    struct CountingProvider {
        calls: Rc<Cell<usize>>,
        entries: BTreeMap<String, String>,
    }

    impl BibliographyProvider for CountingProvider {
        fn name(&self) -> &'static str {
            "InspireHEP"
        }

        fn resolve(&self, key: &str) -> super::Result<Option<ResolvedEntry>> {
            self.calls.set(self.calls.get() + 1);
            Ok(self.entries.get(key).map(|bibtex| ResolvedEntry {
                canonical_id: format!("record-{key}"),
                bibtex: bibtex.clone(),
                provider: self.name(),
            }))
        }
    }

    #[test]
    fn scans_tex_citations_and_bibliography() {
        let dir = tempdir().expect("tempdir");
        let tex = dir.path().join("main.tex");
        std::fs::write(
            &tex,
            "\\citep[e.g.][]{2404.14498, 10.1234/example}\n% \\cite{ignored}\n\\bibliography{refs}",
        )
        .expect("write tex");

        let keys = citation_keys(&[tex]).expect("scan");
        assert!(keys.contains("2404.14498"));
        assert!(keys.contains("10.1234/example"));
        assert!(!keys.contains("ignored"));
    }

    #[test]
    fn syncs_arxiv_key_to_output_bib() {
        let dir = tempdir().expect("tempdir");
        let tex = dir.path().join("main.tex");
        let bib = dir.path().join("refs.bib");
        std::fs::write(&tex, "\\cite{2404.14498}\n\\bibliography{refs}").expect("write tex");
        let provider = FakeProvider {
            entries: BTreeMap::from([(
                "2404.14498".to_owned(),
                "@article{whatever,\n  title = {Example},\n  year = {2024}\n}".to_owned(),
            )]),
        };

        let report = sync_files_with_provider(
            &[tex],
            &SyncOptions {
                output: Some(bib.clone()),
                provider: ProviderChoice::Inspire,
                backup: false,
                ..SyncOptions::default()
            },
            &provider,
        )
        .expect("sync");

        assert_eq!(report.added, vec!["2404.14498"]);
        assert!(
            std::fs::read_to_string(bib)
                .expect("read bib")
                .contains("@article{2404.14498")
        );
    }

    #[test]
    fn check_mode_reports_change_without_writing() {
        let dir = tempdir().expect("tempdir");
        let tex = dir.path().join("main.tex");
        let bib = dir.path().join("refs.bib");
        std::fs::write(&tex, "\\cite{2404.14498}").expect("write tex");
        let provider = FakeProvider {
            entries: BTreeMap::from([(
                "2404.14498".to_owned(),
                "@article{x,\n  title = {Example}\n}".to_owned(),
            )]),
        };

        let report = sync_files_with_provider(
            &[tex],
            &SyncOptions {
                output: Some(bib.clone()),
                check: true,
                backup: false,
                ..SyncOptions::default()
            },
            &provider,
        )
        .expect("sync");

        assert!(report.changed);
        assert!(!bib.exists());
    }

    #[test]
    fn bib_update_mode_uses_existing_keys() {
        let dir = tempdir().expect("tempdir");
        let bib = dir.path().join("refs.bib");
        std::fs::write(&bib, "@article{2404.14498,\n  title = {Old}\n}\n").expect("write bib");
        let provider = FakeProvider {
            entries: BTreeMap::from([(
                "2404.14498".to_owned(),
                "@article{x,\n  title = {New}\n}".to_owned(),
            )]),
        };

        let report = sync_files_with_provider(
            &[PathBuf::from(&bib)],
            &SyncOptions {
                backup: false,
                force_regenerate: true,
                ..SyncOptions::default()
            },
            &provider,
        )
        .expect("sync");

        assert_eq!(report.updated, vec!["2404.14498"]);
        assert!(
            std::fs::read_to_string(bib)
                .expect("read bib")
                .contains("New")
        );
    }

    #[test]
    fn cached_provider_reuses_cached_entry() {
        let dir = tempdir().expect("tempdir");
        let calls = Rc::new(Cell::new(0));
        let provider = CountingProvider {
            calls: Rc::clone(&calls),
            entries: BTreeMap::from([(
                "2404.14498".to_owned(),
                "@article{x,\n  title = {Cached}\n}".to_owned(),
            )]),
        };
        let cached = super::CachedProvider {
            inner: Box::new(provider),
            config: super::CacheConfig {
                enabled: true,
                refresh: false,
                root: dir.path().to_owned(),
            },
        };

        let first = cached
            .resolve_many(&["2404.14498".to_owned()])
            .expect("first resolve");
        assert_eq!(first.len(), 1);
        let second = cached
            .resolve_many(&["2404.14498".to_owned()])
            .expect("second resolve");
        assert_eq!(second.len(), 1);
        assert_eq!(calls.get(), 1);
    }
}
