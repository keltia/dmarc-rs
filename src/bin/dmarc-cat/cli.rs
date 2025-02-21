//! Actual command-line parsing module

// Std library
//
use std::io::{stdin, Read, Write};
use std::path::PathBuf;

// External crates
//
use clap::{crate_authors, crate_description, crate_version, Parser};
use dmarc_rs::{Entry, Input, Status};
use eyre::{bail, eyre};
use tempfile::NamedTempFile;
use tracing::{info, trace};

/// Binary name, using a different binary name
pub(crate) const NAME: &str = env!("CARGO_BIN_NAME");
/// Binary version
pub(crate) const VERSION: &str = crate_version!();
/// Authors
pub(crate) const AUTHORS: &str = crate_authors!();
/// Description
pub(crate) const DESCR: &str = crate_description!();

/// All parsable options and arguments.
#[derive(Parser, Debug)]
#[command(disable_version_flag = true)]
#[clap(version = VERSION, author = AUTHORS, name = NAME, about = DESCR)]
pub struct Opts {
    /// debug mode
    #[clap(short = 'D', long = "debug")]
    pub debug: bool,
    /// Do not resolve IP to names
    #[clap(short = 'N', long = "no-resolve")]
    pub noresolve: bool,
    /// Verbose mode
    #[clap(short = 'v', long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[clap(short = 'q', long)]
    pub quiet: bool,
    /// Display version and exit
    #[clap(short = 'V', long = "version")]
    pub version: bool,
    /// Specify the type of input data
    #[clap(short = 't', long = "input-type", value_parser)]
    pub itype: Option<Input>,
    /// Filenames (possibly none or -)
    pub files: Option<Vec<String>>,
}

/// Build our list of files to be processed later from arguments.
///
#[tracing::instrument]
pub(crate) fn check_args(opts: Opts) -> eyre::Result<Vec<Entry>> {
    // Get arguments
    //
    let flist = opts.files.unwrap_or(vec![]);
    trace!("list={:?}", flist);

    // If no arguments, we assume stdin and we enforce the presence of `-t`.
    //
    let elist = if flist.is_empty() {
        // Assume stdin
        //
        let ft = match opts.itype {
            Some(it) => it,
            None => bail!("-t MUST be provided"),
        };
        info!("only stdin with format {:?}", ft);

        // Save it in a temp file
        //
        let mut p = NamedTempFile::new()?;
        let mut data = String::new();
        let n = stdin().read_to_string(&mut data)?;
        trace!("{n} bytes read from stdin");

        if let Err(e) = p.write(data.as_bytes()) {
            let p = p.path().to_path_buf();
            return Err(Status::WritingFile(p, e.to_string()).into());
        }

        let mut e = Entry::from_str(&format!("{:?}", p.path()));
        e.with(ft);
        vec![e.clone()]
    } else {
        // Otherwise inspect the list and weed out bad files
        //
        info!("Will process: {:?}", flist);

        // Only process files which exist obv
        //
        let (list, badfiles): (Vec<_>, Vec<_>) = flist
            .iter()
            .inspect(|&f| trace!("looking at {:?}", f))
            .partition(|&fname| {
                let fname = PathBuf::from(fname);
                fname.exists() && Input::from(&fname) != Input::Unknown
            });

        let list: Vec<_> = list.iter().map(|fname| Entry::from_str(fname)).collect();

        // Display bad files
        //
        info!("Ignored/bad files: {:?}", badfiles);

        // Do the thing.
        //
        if list.is_empty() {
            return Err(eyre!("Empty file list"));
        }

        dbg!(&list);
        list
    };
    Ok(elist)
}
