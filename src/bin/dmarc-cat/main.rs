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

// Our crates
//
use cli::{Opts,valid_input};
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

    match opts.itype {
        Some(t) => valid_input(t.as_str()),
        _ => panic!("Invalid type")
    };

    Ok(())
}
