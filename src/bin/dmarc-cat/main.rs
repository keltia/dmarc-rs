//! This is the dmarc-cat utility.
//!
//! `dmarc-cat` is a small command-line utility to analyze and display in a usable manner
//! the content of the DMARC XML reports sent by the various email providers around the globe.
//! Should work properly on UNIX (FreeBSD, Linux, etc.) and Windows systems.
//!
//! ## Columns
//!
//! The full XML grammar is available here: [dmarc.xsd](https://tools.ietf.org/html/rfc7489#appendix-C)
//! (for your convenience, a local copy is the `doc/` directory in the repository.
//!
//! The report has several columns:
//!
//! - `IP` is matching IP address
//! - `Count` is the number of times this IP was present
//! - `From` is the `From:` header value
//! - `RFrom` is the envelope `From` value
//! - `RDKIM` is the result from DKIM checking
//! - `RSPF` is the result from SPF checking
//!
//! ## Notes
//!
//! The package is still named `dmarc_rs` to distinguish it from the [Go] version
//! but the binary will remain the same (`dmarc-cat`) and can totally replace it.
//!
//! ## References
//!
//! - [DMARC](https://dmarc.org/)
//! - [DMARC RFC](https://tools.ietf.org/html/rfc7489)
//! - [SPF](http://www.rfc-editor.org/info/rfc7208)
//! - [DKIM](http://www.rfc-editor.org/info/rfc6376)
//!
//! [Go]: https://golang.org/

// Internal crates
//
pub mod cli;
pub mod version;

// Std library

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
        return Ok(())
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
            },
        }
    } else {
        println!("{:?}", opts.files);
    }

    Ok(())
}
