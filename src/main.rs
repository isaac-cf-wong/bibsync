//! Command-line entry point for the template package.

use clap::Parser;
use rust_package_template::greeting;

/// Print a greeting.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Name to greet.
    #[arg(default_value = "world")]
    name: String,
}

fn main() {
    let cli = Cli::parse();

    match greeting(&cli.name) {
        Ok(message) => println!("{message}"),
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(2);
        }
    }
}
