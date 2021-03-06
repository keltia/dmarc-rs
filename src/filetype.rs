//! Input file type handling
//!
//! We allow different type of files as input to `dmarc-cat`:
//!
//! - `plain` for plain XML
//! - `gzip` for gzipped XML
//! - `zip` for Zip files containing both CSV and XML versions
//!
//! We also accept the following aliases:
//!
//! - `txt` for plain files
//! - `gz` for gzip files.
//!
//! The name is not case-sensitive as seen in the tests below.
//!
//! `valid_input()` returns one of the `Input` enum values or an error.
//!

// Std Library
//
use std::ffi::OsStr;
use std::path::Path;

// External crates
//
use anyhow::{anyhow, Result};

/// Allowed type of input
///
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Input {
    /// Plain CSV files
    Csv,
    /// plain text files
    Plain,
    /// XML compressed with gzip
    Gzip,
    /// Actual XML files
    Xml,
    /// ZIP files with generally both CSV and XML
    Zip,
}

/// Validate the input type.
///
/// Our currently supported file types:
///
/// - CSV
/// - GZIP
/// - Zip
/// - XML
/// - Plain text (aka invalid)
///
/// Example:
/// ```rust
/// # use dmarc_rs::filetype::*;
/// let inp = valid_input("plain").unwrap();
/// assert_eq!(Input::Plain, inp);
/// ```
///
pub fn valid_input(itype: &str) -> Result<Input> {
    return match itype.to_lowercase().as_str() {
        "csv" => Ok(Input::Csv),
        "plain" => Ok(Input::Plain),
        "txt" => Ok(Input::Plain),
        "gzip" => Ok(Input::Gzip),
        "gz" => Ok(Input::Gzip),
        "xml" => Ok(Input::Xml),
        "zip" => Ok(Input::Zip),
        _ => Err(anyhow!("Invalid type")),
    };
}

/// Matches a filename to a given input type based on the extension.
/// Assumes stdin/- is plain text unless specified elsewere
///
pub fn ext_to_ftype(p: &Path) -> Input {
    let ext = match p.extension() {
        Some(ext) => ext,
        _ => OsStr::new("txt"),
    }
    .to_owned();
    match ext.into_string().unwrap().to_lowercase().as_str() {
        "csv" => Input::Csv,
        "zip" => Input::Zip,
        "txt" => Input::Plain,
        "xml" => Input::Xml,
        "gz" => Input::Gzip,
        _ => Input::Plain,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::path::PathBuf;

    #[rstest]
    #[case("foo", Input::Plain)]
    #[case("foo.txt", Input::Plain)]
    #[case("foo.zip", Input::Zip)]
    #[case("foo.ZIP", Input::Zip)]
    #[case("foo.gz", Input::Gzip)]
    #[case("foo.GZ", Input::Gzip)]
    #[case("foo.Gz", Input::Gzip)]
    #[case("foo.xml", Input::Xml)]
    #[case("foo.XML", Input::Xml)]
    #[case("foo.csv", Input::Csv)]
    #[case("foo.CSV", Input::Csv)]
    #[case(".CSV", Input::Plain)]
    fn test_ext_to_ftype(#[case] f: PathBuf, #[case] t: Input) {
        let p = PathBuf::from(f);
        assert_eq!(t, ext_to_ftype(&p))
    }

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
            Err(_) => Input::Plain,
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
