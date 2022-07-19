//! This is the `dmarc-cat` utility.
//!
//! `dmarc-cat` is a small command-line utility to analyze and display in a usable manner
//! the content of the DMARC XML reports sent by the various email providers around the globe.
//! Should work properly on UNIX (FreeBSD, Linux, etc.) and Windows systems.
//!
//! ## Usage
//!
//! ```console
//! dmarc-cat 0.2.0
//! Ollivier Robert <roberto@keltia.net>
//! Rust utility to decode and display DMARC reports.
//!
//! USAGE:
//!     dmarc-cat [OPTIONS] [FILES]...
//!
//! ARGS:
//!     <FILES>...    Filenames (possibly none or -)
//!
//! OPTIONS:
//!     -D, --debug                 debug mode
//!     -h, --help                  Print help information
//!     -j, --jobs <JOBS>           Use this many parallel jobs for resolving IP [default: 6]
//!     -N, --no-resolve            Do not resolve IP to names
//!     -t, --input-type <ITYPE>    Specify the type of input data
//!     -v, --verbose               Verbose mode
//!     -V, --version               Display version and exit
//! ```
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

// Internal crates
//
pub mod analyze;
pub mod async_resolve;
pub mod cli;
pub mod file;
pub mod version;

// Std library
//

// Our crates
//
use cli::Opts;
use resolve::{parallel_solve, simple_solve};
use dmarc_rs::filetype::*;
use dmarc_rs::ip::Ip;
use dmarc_rs::iplist::IpList;
use dmarc_rs::resolver::{res_init, ResType, Solver};
use file::{check_for_files, scan_list};
use version::version;

// External crates
//
use anyhow::{anyhow, Result};
use clap::Parser;

/// Main entry point
///
fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    // By-pass everything
    if opts.version {
        println!("{}", version());
        return Ok(());
    }

    let mut flist = opts.files.to_owned();
    let mut ftype = Input::Plain;

    // Handle --no-resolv flag
    //
    let mut res = res_init(ResType::Real);
    if opts.noresolve {
        res = res_init(ResType::Null);
    }

    // If no arguments, we assume stdin and we enforece the presence of `-t`.
    //
    if flist.is_empty() {
        // Assume stdin
        ftype = match opts.itype {
            Some(it) => match valid_input(&it) {
                Ok(it) => it,
                _ => return Err(anyhow!("Invalid type for -t")),
            },
            None => return Err(anyhow!("-t MUST be provided")),
        };
        flist.push("-".into())
    }

    println!("{:?}", flist);

    // Check each file in the list and returns only the valid ones
    //
    let flist = check_for_files(&flist);
    if flist.is_empty() {
        return Err(anyhow!("No valid files"));
    }

    // Do the thing.
    //
    let output = match scan_list(&flist) {
        Ok(res) => res,
        Err(e) => {
            format!("Error: {:?}", e)
        }
    };
    println!("{:?}", output);
    Ok(())
}
