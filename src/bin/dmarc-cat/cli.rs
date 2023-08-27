//! Actual command-line parsing module

// Std library
//

// External crates
//
use clap::{crate_authors, crate_description, crate_version, Parser};

use dmarc_rs::Input;

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
