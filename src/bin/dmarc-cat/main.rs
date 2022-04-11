//! This is the dmarc-cat utility.
//!
//! The package is still named `dmarc-rs` to distinguish it from the Go version
//! but the binary will remain the same.

// Internal crates
//
mod cli;

use cli::{version, Opts};

// External crates
//
use anyhow::Result;
use clap::Parser;

/// Main entry point
///
fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    if opts.version {
        version()
    }

    Ok(())
}
