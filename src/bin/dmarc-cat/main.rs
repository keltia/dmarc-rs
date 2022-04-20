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
use std::env::args;
use std::process::exit;

// Our crates
//
use cli::{valid_input, Input, Opts};
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

    let mut ftype = Input::Plain;

    // If no arguments or argument == "-"
    //
    if opts.files.is_empty() {
        // Assume stdin
        match opts.itype {
            None => panic!("-t MUST be provided"),
            Some(it) => match valid_input(&it) {
                Ok(it) => ftype = it,
                _ => panic!("Invalid type for -t"),
            }
        }
    } else {
        println!("{:?}", opts.files);
    }

    Ok(())
}
