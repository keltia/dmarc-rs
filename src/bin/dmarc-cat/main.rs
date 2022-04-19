//! This is the dmarc-cat utility.
//!
//! The package is still named `dmarc-rs` to distinguish it from the Go version
//! but the binary will remain the same.

// Internal crates
//
mod cli;
mod version;

// Std library
use std::process::exit;

use cli::Opts;
use version::version;

// External crates
//
use anyhow::Result;
use clap::Parser;

/// Main entry point
///
fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    // By-pass everything
    if opts.version {
        println!("{}", version());
        exit(1)
    }

    Ok(())
}
