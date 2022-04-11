//! versioning stuff

use clap::{crate_authors, crate_description, crate_version};

/// Binary name, using a different binary name
pub(crate) const NAME: &str = env!("CARGO_BIN_NAME");
/// Binary version
pub(crate) const VERSION: &str = crate_version!();
/// Authors
pub(crate) const AUTHORS: &str = crate_authors!();

/// Display our version banned
pub fn version() {
    println!(
        "{}/{}/j{} by {}\n{}",
        NAME,
        VERSION,
        num_cpus::get_physical(),
        AUTHORS,
        crate_description!()
    );
}
