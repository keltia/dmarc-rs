//! Actual command-line parsing stuff

use anyhow::{anyhow, Result};
use clap::{crate_authors, crate_description, crate_version, AppSettings, Parser};

use crate::version::NAME;

/// Help message
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
    /// Use this many parallel jobs for resolving IP
    #[clap(short = 'j', long = "jobs", default_value_t = num_cpus::get_physical())]
    pub jobs: usize,
    /// Specify the type of input data
    #[clap(short = 't', long = "input-type")]
    pub itype: Option<String>,
}

/// Allowed type of input
///
#[derive(Debug, PartialEq)]
pub enum Input {
    Plain,
    Gzip,
    Zip,
}

/// Validate the input type through the -t option
///
/// Example:
/// ```
/// # use dmarc_rs::cli::valid_input;
///
/// let inp = valid_input("plain");
/// assert_eq!(Input::Plain, inp);
/// ```
///
pub fn valid_input(itype: &str) -> Result<Input> {
    return match itype.to_lowercase().as_str() {
        "plain" => Ok(Input::Plain),
        "txt" => Ok(Input::Plain),
        "gzip" => Ok(Input::Gzip),
        "gz" => Ok(Input::Gzip),
        "zip" => Ok(Input::Zip),
        _ => Err(anyhow!("Invalid type")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("plain", Input::Plain)]
    #[case("TXT", Input::Plain)]
    #[case("gzip", Input::Gzip)]
    #[case("gz", Input::Gzip)]
    #[case("zip", Input::Zip)]
    #[case("Zip", Input::Zip)]
    fn test_valid_input_ok(#[case] s: &str, #[case] it: Input) {
        let r = valid_input(s);
        assert!(valid_input(s).is_ok());
        let r = match r {
            Ok(r) => r,
            Err(_) => panic!("nok"),
        };
        assert_eq!(it, r);
    }

    #[rstest]
    #[case("")]
    #[case("qZip")]
    #[case("")]
    fn test_valid_input_nok(#[case] s: &str) {
        let r = valid_input(s);
        assert!(valid_input(s).is_err());
        let r = match r {
            Ok(_) => "bad ok".to_string(),
            Err(e) => e.to_string(),
        };
        assert_eq!("Invalid type", r);
    }

}
