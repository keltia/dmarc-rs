//! This is the `dmarc-cat` utility.
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
//! ## References
//!
//! - [DMARC](https://dmarc.org/)
//! - [DMARC RFC](https://tools.ietf.org/html/rfc7489)
//! - [SPF](http://www.rfc-editor.org/info/rfc7208)
//! - [DKIM](http://www.rfc-editor.org/info/rfc6376)
//!

use clap::Parser;
use eyre::Result;
use rayon::prelude::*;
use tracing::info;

// Our crates
//
use cli::Opts;

use cli::check_args;
use init::init_runtime;
use version::version;

// Internal crates
//
pub mod analyze;
pub mod cli;
mod init;
pub mod version;

/// Main entry point
///
fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    // By-pass everything
    //
    if opts.version {
        eprintln!("{}", version());
        return Ok(());
    }

    let _ = init_runtime(&opts)?;

    let elist = check_args(opts)?;

    info!("{:?} files to be processed", elist);

    // Use rayon to do stuff in parallel
    //
    let output = elist
        .par_iter()
        .map(|e| e.fetch().unwrap())
        .collect::<Vec<String>>()
        .join("\n--\n");
    println!("{:?}", output);

    Ok(())
}
