//! Minimal example for calling the library API.

use bibsync::greeting;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", greeting("example")?);
    Ok(())
}
