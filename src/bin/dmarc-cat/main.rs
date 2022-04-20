//! This is the dmarc-cat utility.
//!
//! ## Notes
//!
//! The package is still named `dmarc-rs` to distinguish it from the [Go] version
//! but the binary will remain the same (`dmarc-cat`) and can totally replace it.
//!
//! ## References
//
//! - [DMARC](https://dmarc.org/)
//! - [SPF](http://www.rfc-editor.org/info/rfc7208)
//! - [DKIM](http://www.rfc-editor.org/info/rfc6376)
//! - [Go]:(https://golang.org/)


// Internal crates
//
mod cli;
mod version;

// Std library
use std::process::exit;

// Our crates
//
use cli::{valid_input, Opts};
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
        _ => panic!("Invalid type"),
    };

    Ok(())
}
