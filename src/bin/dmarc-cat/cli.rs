//! Actual command-line parsing module

// Std library
//
use std::path::PathBuf;

// External crates
//
use clap::{crate_authors, crate_description, crate_version, AppSettings, Parser};

// Internal crates
//
use crate::version::NAME;

/// All parsable options and arguments.
#[derive(Parser, Debug)]
#[clap(name = NAME, about = crate_description!())]
#[clap(version = crate_version!(), author = crate_authors!())]
#[clap(setting = AppSettings::NoAutoVersion)]
pub struct Opts {
    /// debug mode
    #[clap(short = 'D', long = "debug")]
    pub debug: bool,
    /// Do not resolve IP to names
    #[clap(short = 'N', long = "no-resolve")]
    pub noresolve: bool,
    /// Verbose mode
    #[clap(short = 'v', long)]
    pub verbose: bool,
    /// Display version and exit
    #[clap(short = 'V', long = "version")]
    pub version: bool,
    /// Specify the type of input data
    #[clap(short = 't', long = "input-type")]
    pub itype: Option<String>,
    /// Filenames (possibly none or -)
    pub files: Vec<PathBuf>,
}
