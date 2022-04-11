
use clap::{crate_version, crate_authors, Parser,AppSettings};

/// Binary name, using a different binary name
pub(crate) const NAME: &str = env!("CARGO_BIN_NAME");
/// Binary version
pub(crate) const VERSION: &str = crate_version!();
/// Authors
pub(crate) const AUTHORS: &str = crate_authors!();

/// Help message
#[derive(Parser, Debug)]
#[clap(name = "dmarc-cat", about = "Explore DMARC reports")]
#[clap(version = crate_version!(), author = crate_authors!())]
#[clap(setting = AppSettings::NoAutoVersion)]
pub struct Opts {
    /// debug mode
    #[clap(short = 'D', long = "debug")]
    pub debug: Option<bool>,
    /// Do not resolve IP to names
    #[clap(short='N', long="no-resolve")]
    pub noresolve: bool,
    /// Verbose mode
    #[clap(short = 'v', long)]
    pub verbose: bool,
    /// Display version and exit
    #[clap(short = 'V', long = "version")]
    pub version: bool,
    /// Use this many parallel jobs for resolving IP
    #[clap(short='j', long="jobs")]
    pub jobs: Option<usize>,
    /// Specify the type of input data
    #[clap(short='t', long="input-type")]
    pub itype: Option<String>,
}

/// Display our version banned
pub fn version() {
    println!("{}/{} by {}", NAME, VERSION, AUTHORS);
}