//! Synchronize BibTeX files from citation keys in TeX sources.
//!
//! The library scans TeX files for citation keys, resolves identifier-like keys
//! through NASA ADS and/or `InspireHEP`, and merges the resulting BibTeX entries
//! into a target bibliography.

use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
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
    /// Update existing entries when a provider can resolve them.
    pub update_existing: bool,
    /// Regenerate existing entries even when their resolved identifier is unchanged.
    pub force_regenerate: bool,
    /// Copy entries from read-only bibliography files into the output file.
    pub merge_other: bool,
    /// Create `<output>.bak` before overwriting an existing output file.
    pub backup: bool,
    /// Compare only; do not write. Pre-commit hooks can use this for validation.
    pub check: bool,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            output: None,
            other_bibliographies: Vec::new(),
            provider: ProviderChoice::Auto,
            update_existing: true,
            force_regenerate: false,
            merge_other: false,
            backup: true,
            check: false,
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
}

/// Synchronize a bibliography from TeX or BibTeX input files.
///
/// # Errors
///
/// Returns an error when files cannot be read/written, a requested provider
/// cannot be configured, or a provider request fails.
pub fn sync_files(files: &[PathBuf], options: &SyncOptions) -> Result<SyncReport> {
    let provider = ProviderChain::from_choice(options.provider)?;
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

    for key in keys {
        let exists = bibliography.contains(&key);
        let exists_in_other = other_bibliography.contains(&key);

        if exists && !options.update_existing {
            report.existing.push(key);
            continue;
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

        let should_resolve = !exists || options.update_existing;
        if !should_resolve {
            report.existing.push(key);
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

        match provider.resolve(&key)? {
            Some(resolved) => {
                let mut entry = BibEntry::parse(&resolved.bibtex).ok_or_else(|| {
                    BibsyncError::InvalidProviderBibtex {
                        provider: resolved.provider,
                        key: key.clone(),
                    }
                })?;
                entry.key.clone_from(&key);
                bibliography.upsert(entry);
                if exists {
                    if options.force_regenerate || options.update_existing {
                        report.updated.push(key);
                    } else {
                        report.existing.push(key);
                    }
                } else {
                    report.added.push(key);
                }
            }
            None => {
                if exists {
                    report.existing.push(key);
                } else {
                    report.unresolved.push(key);
                }
            }
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
    fn from_choice(choice: ProviderChoice) -> Result<Self> {
        let providers: Vec<Box<dyn BibliographyProvider>> = match choice {
            ProviderChoice::Auto => {
                let mut providers: Vec<Box<dyn BibliographyProvider>> = Vec::new();
                if let Some(ads) = AdsProvider::from_env_optional()? {
                    providers.push(Box::new(ads));
                }
                providers.push(Box::new(InspireProvider::new()?));
                providers
            }
            ProviderChoice::Ads => vec![Box::new(AdsProvider::from_env()?)],
            ProviderChoice::Inspire => vec![Box::new(InspireProvider::new()?)],
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

    fn export_bibtex(&self, bibcode: &str) -> Result<Option<String>> {
        let response: AdsExportResponse = self
            .client
            .post("https://api.adsabs.harvard.edu/v1/export/bibtex")
            .headers(self.headers()?)
            .json(&json!({ "bibcode": [bibcode] }))
            .send()?
            .error_for_status()?
            .json()?;
        Ok(nonempty(&response.export))
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
        let Some(bibtex) = self.export_bibtex(&bibcode)? else {
            return Ok(None);
        };
        Ok(Some(ResolvedEntry {
            canonical_id: bibcode,
            bibtex,
            provider: self.name(),
        }))
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
#[must_use]
pub fn pre_commit_hook_manifest() -> &'static str {
    r"- id: bibsync
  name: bibsync
  description: Synchronize BibTeX entries from TeX citation keys
  entry: bibsync
  language: rust
  types_or: [tex, bib]
"
}

#[cfg(test)]
mod tests {
    use super::{
        BibliographyProvider, ProviderChoice, ResolvedEntry, SyncOptions, citation_keys,
        sync_files_with_provider,
    };
    use std::collections::BTreeMap;
    use std::path::PathBuf;
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
}
