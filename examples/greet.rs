//! Minimal example for calling the library API.

use bibsync::{ProviderChoice, SyncOptions};

fn main() {
    let options = SyncOptions {
        provider: ProviderChoice::Inspire,
        check: true,
        ..SyncOptions::default()
    };
    println!("check mode: {}", options.check);
}
