//! Minimal example for calling the library API.

use rust_package_template::greeting;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", greeting("example")?);
    Ok(())
}
