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

use anyhow::{anyhow, bail, Result};
use clap::Parser;
use log::{debug, error, info, trace, warn};
use std::path::PathBuf;
use stderrlog::LogLevelNum::{Debug, Error, Info, Trace};

// Our crates
//
use cli::Opts;
use dmarc_rs::entry::Entry;
use dmarc_rs::filetype::*;
use dmarc_rs::res::{res_init, ResType};
use dmarc_rs::types::Feedback;
use file::{check_for_files, scan_list};
use version::version;

// External crates
//
use crate::file::handle_one_file;

// Internal crates
//
pub mod analyze;
pub mod cli;
pub mod file;
pub mod version;

use std::fs::File;
// Std library
//
use std::io::{stdin, BufReader};


// External crates
//
use dmarc_rs::task::Task;

/// Main entry point
///
fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    // By-pass everything
    //
    if opts.version {
        println!("{}", version());
        return Ok(());
    }

    // Check verbosity
    //
    let mut lvl = match opts.verbose {
        0 => Info,
        1 => Error,
        2 => Debug,
        3 => Trace,
        _ => Trace,
    };

    if opts.debug {
        lvl = Debug;
    }

    // Prepare logging.
    //
    stderrlog::new()
        .modules(["dmarc-cat", "dmarc-rs"])
        .verbosity(lvl)
        .quiet(opts.quiet)
        .init()?;

    let mut flist = opts.files.to_owned();
    let mut ftype = Input::Plain;

    trace!("list={:?}", flist);

    // Handle --no-resolv flag
    //
    let res = if opts.noresolve {
        info!("noresolv");
        res_init(ResType::Null)
    } else {
        info!("regular resolver");
        res_init(ResType::Real)
    };

    // If no arguments, we assume stdin and we enforce the presence of `-t`.
    //
    let mut current = if flist.is_empty() {
        // Assume stdin
        //
        let ft = match opts.itype {
            Some(it) => match valid_input(&it) {
                Ok(it) => it,
                _ => return bail!("Invalid type for -t"),
            },
            None => return bail!("-t MUST be provided"),
        };
        info!("only stdin with format {:?}", ftype);
        vec![Entry { p: "-".into(), ft }]
        Task::from_reader(stdin(), ftype)
    } else {
        let flist = flist
            .iter()
            // weed out unknown files
            .filter(|p| PathBuf::from(p).exists())
            .map(|p| &PathBuf::from(p))
            // Create en entry with file type
            .collect::<Vec<_>>();
        // Return a single "file" representing stdin
        //


        // Otherwise inspect the list and weed out bad files
        //
        info!("Will process: {:?}", opts.files);

        let (list, failed): (Vec<_>, Vec<_>) = opts
            .files
            .iter()
            .inspect(|f| debug!("looking at {:?}", f))
            .partition(|&fname| fname.exists());

        // Do the thing.
        //
            if list.is_empty() {
            return Err(anyhow!("Empty file list"));
        }

        dbg!(&list);
        Task::from_list(list)
    };
    trace!("task={:?}", current);

    info!("{:?} files to be processed", &current.list());
    let output = current.run()?;
    output.iter().map(|r| {});
    println!("{:?}", output);
    Ok(())
}
