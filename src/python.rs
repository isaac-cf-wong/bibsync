//! Python bindings for `bibsync`.

use crate::{ProviderChoice, SyncOptions, UpdateMode, cli, sync_files};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::path::PathBuf;

fn parse_provider(value: &str) -> PyResult<ProviderChoice> {
    match value {
        "auto" => Ok(ProviderChoice::Auto),
        "ads" => Ok(ProviderChoice::Ads),
        "inspire" => Ok(ProviderChoice::Inspire),
        other => Err(PyValueError::new_err(format!(
            "provider must be 'auto', 'ads', or 'inspire', got {other:?}"
        ))),
    }
}

fn parse_update_mode(value: &str) -> PyResult<UpdateMode> {
    match value {
        "preprints-only" | "preprints_only" => Ok(UpdateMode::PreprinsOnly),
        "never" => Ok(UpdateMode::Never),
        "always" => Ok(UpdateMode::Always),
        other => Err(PyValueError::new_err(format!(
            "update_mode must be 'preprints-only', 'never', or 'always', got {other:?}"
        ))),
    }
}

fn pathbufs(paths: Option<Vec<String>>) -> Vec<PathBuf> {
    paths
        .unwrap_or_default()
        .into_iter()
        .map(PathBuf::from)
        .collect()
}

#[allow(clippy::fn_params_excessive_bools, clippy::too_many_arguments)]
#[pyfunction(name = "sync_files")]
#[pyo3(signature = (
    files,
    output = None,
    other_bibliographies = None,
    provider = "auto",
    update_mode = "preprints-only",
    force_regenerate = false,
    merge_other = false,
    backup = true,
    check = true,
    cache = false,
    refresh_cache = false,
    cache_dir = None,
    ignore_file = None
))]
fn sync_files_py<'py>(
    py: Python<'py>,
    files: Vec<String>,
    output: Option<String>,
    other_bibliographies: Option<Vec<String>>,
    provider: &str,
    update_mode: &str,
    force_regenerate: bool,
    merge_other: bool,
    backup: bool,
    check: bool,
    cache: bool,
    refresh_cache: bool,
    cache_dir: Option<String>,
    ignore_file: Option<String>,
) -> PyResult<Bound<'py, PyDict>> {
    let options = SyncOptions {
        output: output.map(PathBuf::from),
        other_bibliographies: pathbufs(other_bibliographies),
        provider: parse_provider(provider)?,
        update_mode: parse_update_mode(update_mode)?,
        force_regenerate,
        merge_other,
        backup,
        check,
        cache,
        refresh_cache,
        cache_dir: cache_dir.map(PathBuf::from),
        ignore_file: ignore_file.map(PathBuf::from),
    };
    let files = files.into_iter().map(PathBuf::from).collect::<Vec<_>>();
    let report =
        sync_files(&files, &options).map_err(|error| PyValueError::new_err(error.to_string()))?;

    let dict = PyDict::new(py);
    dict.set_item("output", report.output.to_string_lossy().as_ref())?;
    dict.set_item("added", report.added)?;
    dict.set_item("updated", report.updated)?;
    dict.set_item("existing", report.existing)?;
    dict.set_item("found_in_other", report.found_in_other)?;
    dict.set_item("unresolved", report.unresolved)?;
    dict.set_item("changed", report.changed)?;
    dict.set_item("check_mode", report.check_mode)?;
    Ok(dict)
}

#[pyfunction]
fn run_cli(args: Vec<String>) -> i32 {
    cli::run_cli_from(args)
}

#[pymodule]
fn _bibsync(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(sync_files_py, module)?)?;
    module.add_function(wrap_pyfunction!(run_cli, module)?)?;
    Ok(())
}
