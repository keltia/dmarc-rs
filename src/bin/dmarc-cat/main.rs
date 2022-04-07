//! This is the dmarc-cat utility.
//!
//! The package is still named `dmarc-rs`to distinguish it from the Go version
//! but the binary will remain the same.

// External crates
//
use anyhow::Result;
use clap::crate_version;

/// Binary name, using a different binary name
pub(crate) const NAME: &str = "dmarc-cat";
/// Binary version
pub(crate) const VERSION: &str = crate_version!();

/// Main entry point
///
fn main() -> Result<()> {
    println!("{}/{}", NAME, VERSION);
    Ok(())
}
