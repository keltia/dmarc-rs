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

// External crates
//
use anyhow::{anyhow, Result};

/// Allowed type of input
///
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum Input {
    /// plain text files aka utf-8 XML
    Plain,
    /// XML compressed with gzip
    Gzip,
    /// ZIP files with generally both CSV and XML
    Zip,
}

/// Validate the input type.
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
        "plain" => Ok(Input::Plain),
        "txt" => Ok(Input::Plain),
        "gzip" => Ok(Input::Gzip),
        "gz" => Ok(Input::Gzip),
        "zip" => Ok(Input::Zip),
        _ => Err(anyhow!("Invalid type")),
    };
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
