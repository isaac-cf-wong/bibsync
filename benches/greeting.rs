//! Benchmark target for the sample greeting helper.

fn main() {
    divan::main();
}

#[divan::bench]
fn greeting() {
    let _ = rust_package_template::greeting("benchmark");
}
