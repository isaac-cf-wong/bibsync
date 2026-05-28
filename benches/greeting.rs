//! Benchmark target for TeX citation scanning.

fn main() {
    divan::main();
}

#[divan::bench]
fn pre_commit_manifest() {
    let _ = bibsync::pre_commit_hook_manifest(None);
}
