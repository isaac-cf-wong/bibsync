//! Command-line entry point for `bibsync`.

fn main() {
    std::process::exit(bibsync::cli::run_cli());
}
